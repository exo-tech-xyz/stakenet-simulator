// TODO: For each validator load a stake account that has a long history

use std::{collections::HashMap, str::FromStr};

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use sqlx::{Pool, Postgres};
use stakenet_simulator_db::{
    inflation_rewards::InflationReward, stake_accounts::StakeAccount,
    validator_history_entry::ValidatorHistoryEntry,
};
use tracing::info;

use crate::{EpochRewardsTrackerError, rpc_utils};

/// Uses inflation_rewards data in the DB and the validator history (for total active stake) to
/// calculate and update a validator's inflation
pub async fn gather_total_inflation_rewards_per_epoch(
    db_connection: &Pool<Postgres>,
) -> Result<(), EpochRewardsTrackerError> {
    let vote_keys = ValidatorHistoryEntry::get_all_vote_pubkeys(db_connection).await?;
    // TODO: Parallelize
    for vote_pubkey in vote_keys {
        // Gather the valdiator history entries
        let validator_history_entries =
            ValidatorHistoryEntry::fetch_by_validator(db_connection, &vote_pubkey).await?;
        let epoch_validator_history_entry_map: HashMap<u64, ValidatorHistoryEntry> =
            validator_history_entries
                .into_iter()
                .map(|x| (u64::from(x.validator_history_entry.epoch), x))
                .collect();
        // fetch all the inflation rewards related to the validator
        let infation_rewards =
            InflationReward::fetch_by_validator(db_connection, &vote_pubkey).await?;
        let mut epoch_inflation_rewards_map: HashMap<u64, Vec<InflationReward>> = HashMap::new();
        for inflation_reward in infation_rewards.into_iter() {
            match epoch_inflation_rewards_map.get_mut(&inflation_reward.epoch) {
                Some(prev_val) => {
                    prev_val.push(inflation_reward);
                }
                None => {
                    epoch_inflation_rewards_map
                        .insert(inflation_reward.epoch, vec![inflation_reward]);
                }
            }
        }

        // Run the calclulations for each epoch
        for (epoch, history_entry) in epoch_validator_history_entry_map.iter() {
            match epoch_inflation_rewards_map.get(epoch) {
                Some(inflation_rewards) => {
                    let mut stake_to_reward_ratios: Vec<f64> = Vec::new();
                    for inflation_reward in inflation_rewards {
                        let stake_amount = inflation_reward.post_balance - inflation_reward.amount;

                        stake_to_reward_ratios.push(inflation_reward.amount as f64 / stake_amount as f64);
                        let commission = inflation_reward
                            .commission
                            .map(|x| u8::try_from(x).unwrap());
                        let total_inflation_rewards = calculate_total_inflation_rewards(
                            history_entry
                                .validator_history_entry
                                .activated_stake_lamports,
                            stake_amount,
                            commission,
                            inflation_reward.amount,
                        );
                        // info!(
                        //     "Validator {} received {} inflation rewards for epoch {}. {} | {} | {} | {}",
                        //     vote_pubkey,
                        //     total_inflation_rewards,
                        //     epoch,
                        //     history_entry
                        //         .validator_history_entry
                        //         .activated_stake_lamports,
                        //     stake_amount,
                        //     commission.unwrap_or(0),
                        //     inflation_reward.amount,
                        // );
                    }
                    info!("Rewards/Stake ratios: {:?}", stake_to_reward_ratios);
                }
                None => {}
            }
        }
    }
    Ok(())
}

// TODO (nice to have): Parllelize these batches and calls
pub async fn gather_inflation_rewards(
    db_connection: &Pool<Postgres>,
    rpc_client: &RpcClient,
) -> Result<(), EpochRewardsTrackerError> {
    // Fetch all the stake_accounts from the DB
    let stake_account_keys = StakeAccount::get_all_pubkeys(db_connection).await?;
    let stake_account_keys: Vec<Pubkey> = stake_account_keys
        .into_iter()
        .map(|x| Pubkey::from_str(&x).unwrap())
        .collect();
    // Break them into chunks of 30 (335 batches)
    for stake_accounts in stake_account_keys.chunks(30).into_iter() {
        // For each batch, call getInflationRewards for epochs >= 700 (120 calls)
        info!(
            "Fetching inflation rewards for stake accounts {:?}",
            stake_accounts
        );
        for epoch in 700u64..818 {
            let rewards =
                rpc_utils::get_inflation_rewards(rpc_client, stake_accounts, epoch).await?;
            // Build a set of records and insert into the DB
            let records: Vec<InflationReward> = rewards
                .into_iter()
                .zip(stake_accounts)
                .filter_map(|(maybe_inflation_reward, stake_account)| {
                    Some(InflationReward::from_rpc_inflation_reward(
                        maybe_inflation_reward?,
                        stake_account,
                    ))
                })
                .collect();
            InflationReward::bulk_insert(db_connection, records).await?;
        }
    }

    Ok(())
}

pub async fn get_inflation_rewards(
    db_connection: &Pool<Postgres>,
    rpc_client: &RpcClient,
) -> Result<(), EpochRewardsTrackerError> {
    let epoch = 801;
    let vote_pubkey = pubkey!("6q1VNp8Vy2Go12vb8CwbjUqqj2SXr2JYftJRWs71sW23");
    let addresses = vec![pubkey!("2KxnNM2TEtUWYvsxhFk4qn3ix5CBohaXFVAzhn8iMuCS")];
    let res = ValidatorHistoryEntry::fetch_by_validator_and_epoch(
        db_connection,
        &vote_pubkey.to_string(),
        epoch,
    )
    .await?
    .expect("result from DB");
    let rewards = crate::rpc_utils::get_inflation_rewards(rpc_client, &addresses, epoch).await?;

    for reward in rewards.into_iter() {
        let account_rewards = reward.unwrap();
        let pre_balance = account_rewards.post_balance - account_rewards.amount;
        let total_inflation_rewards = calculate_total_inflation_rewards(
            res.validator_history_entry.activated_stake_lamports,
            pre_balance,
            account_rewards.commission,
            account_rewards.amount,
        );
    }

    Ok(())
}

pub fn calculate_total_inflation_rewards(
    total_active_stake: u64,
    stake_amount: u64,
    commission: Option<u8>,
    inflation_rewards: u64,
) -> u64 {
    // First we get factor in the commission rate to get the calculated total inflation rewards
    //  attributed to this stake account. When commission is unknown or 0, then this is the inflation_rewards.
    //  When there's a commission it's `inflation_reards * 100 / commission`` which  assumes 
    //  rewards are pro-rated evenly across active stake.
    let total_rewards_for_stake_account = if let Some(commission) = commission {
        if commission == 0 {
            inflation_rewards
        } else {
            u128::from(inflation_rewards)
                .checked_mul(100)
                .and_then(|x| x.checked_div(u128::from(commission)))
                .unwrap() as u64
        }
    } else {
        inflation_rewards
    };
    // Factor in the stake_accounts rewards amount for it's stake, relative to the 
    // total_active_stake on the validator
    u128::from(total_rewards_for_stake_account)
        .checked_mul(u128::from(total_active_stake))
        .and_then(|x: u128| x.checked_div(u128::from(stake_amount)))
        .unwrap() as u64
}
