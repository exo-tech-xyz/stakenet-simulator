// TODO: For each validator load a stake account that has a long history

use std::str::FromStr;

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
        info!("Fetching inflation rewards for stake accounts {:?}", stake_accounts);
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
    let total_rewards_for_stake_account = if let Some(commission) = commission {
        inflation_rewards
            .checked_mul(100)
            .and_then(|x| x.checked_div(u64::from(commission)))
            .unwrap()
    } else {
        inflation_rewards
    };
    u128::from(total_rewards_for_stake_account)
        .checked_mul(u128::from(total_active_stake))
        .and_then(|x| x.checked_div(u128::from(stake_amount)))
        .unwrap() as u64
}
