use std::collections::HashMap;

use clap::Parser;
use futures::stream::StreamExt;
use jito_steward::{Config, constants::TVC_ACTIVATION_EPOCH, score::validator_score};
use num_traits::cast::ToPrimitive;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use sqlx::{Pool, Postgres};
use stakenet_simulator_db::{
    cluster_history::ClusterHistory, cluster_history_entry::ClusterHistoryEntry,
    epoch_rewards::EpochRewards, validator_history::ValidatorHistory,
    validator_history_entry::ValidatorHistoryEntry,
};
use tracing::{error, info};
use validator_history::ClusterHistory as JitoClusterHistory;

use crate::{error::CliError, modify_config_parameter_from_args, steward_utils::fetch_config};

const DAYS_PER_YEAR: f64 = 365.0;

#[derive(Clone, Debug, Parser)]
pub struct BacktestArgs {
    #[arg(long, env)]
    pub mev_commission_range: Option<u16>,
    #[arg(long, env)]
    pub epoch_credits_range: Option<u16>,
    #[arg(long, env)]
    pub commission_range: Option<u16>,
    #[arg(long, env)]
    pub scoring_delinquency_threshold_ratio: Option<f64>,
    #[arg(long, env)]
    pub instant_unstake_delinquency_threshold_ratio: Option<f64>,
    #[arg(long, env)]
    pub mev_commission_bps_threshold: Option<u16>,
    #[arg(long, env)]
    pub commission_threshold: Option<u8>,
    #[arg(long, env)]
    pub historical_commission_threshold: Option<u8>,
    #[arg(long, env)]
    pub priority_fee_lookback_epochs: Option<u8>,
    #[arg(long, env)]
    pub priority_fee_lookback_offset: Option<u8>,
    #[arg(long, env)]
    pub priority_fee_max_commission_bps: Option<u16>,
    #[arg(long, env)]
    pub priority_fee_error_margin_bps: Option<u16>,
    #[arg(long, env)]
    pub num_delegation_validators: Option<u32>,
    #[arg(long, env)]
    pub scoring_unstake_cap_bps: Option<u32>,
    #[arg(long, env)]
    pub instant_unstake_cap_bps: Option<u32>,
    #[arg(long, env)]
    pub stake_deposit_unstake_cap_bps: Option<u32>,
    #[arg(long, env)]
    pub instant_unstake_epoch_progress: Option<f64>,
    #[arg(long, env)]
    pub compute_score_slot_range: Option<u64>,
    #[arg(long, env)]
    pub instant_unstake_inputs_epoch_progress: Option<f64>,
    #[arg(long, env)]
    pub num_epochs_between_scoring: Option<u64>,
    #[arg(long, env)]
    pub minimum_stake_lamports: Option<u64>,
    #[arg(long, env)]
    pub minimum_voting_epochs: Option<u64>,
    #[arg(long, env)]
    priority_fee_scoring_start_epoch: Option<u16>,
    #[arg(long, env)]
    target_epoch: Option<u64>,
}

impl BacktestArgs {
    pub fn update_steward_config(&self, config: &mut Config) {
        modify_config_parameter_from_args!(self, config, mev_commission_range);
        modify_config_parameter_from_args!(self, config, epoch_credits_range);
        modify_config_parameter_from_args!(self, config, commission_range);
        modify_config_parameter_from_args!(self, config, scoring_delinquency_threshold_ratio);
        modify_config_parameter_from_args!(
            self,
            config,
            instant_unstake_delinquency_threshold_ratio
        );
        modify_config_parameter_from_args!(self, config, mev_commission_bps_threshold);
        modify_config_parameter_from_args!(self, config, commission_threshold);
        modify_config_parameter_from_args!(self, config, historical_commission_threshold);
        modify_config_parameter_from_args!(self, config, priority_fee_lookback_epochs);
        modify_config_parameter_from_args!(self, config, priority_fee_lookback_offset);
        modify_config_parameter_from_args!(self, config, priority_fee_max_commission_bps);
        modify_config_parameter_from_args!(self, config, priority_fee_error_margin_bps);
        modify_config_parameter_from_args!(self, config, num_delegation_validators);
        modify_config_parameter_from_args!(self, config, scoring_unstake_cap_bps);
        modify_config_parameter_from_args!(self, config, instant_unstake_cap_bps);
        modify_config_parameter_from_args!(self, config, stake_deposit_unstake_cap_bps);
        modify_config_parameter_from_args!(self, config, compute_score_slot_range);
        modify_config_parameter_from_args!(self, config, instant_unstake_epoch_progress);
        modify_config_parameter_from_args!(self, config, instant_unstake_inputs_epoch_progress);
        modify_config_parameter_from_args!(self, config, num_epochs_between_scoring);
        modify_config_parameter_from_args!(self, config, minimum_stake_lamports);
        modify_config_parameter_from_args!(self, config, minimum_voting_epochs);
        modify_config_parameter_from_args!(self, config, priority_fee_scoring_start_epoch);
    }
}

pub async fn handle_backtest(
    args: BacktestArgs,
    db_connection: &Pool<Postgres>,
    rpc_client: &RpcClient,
) -> Result<(), CliError> {
    // TODO: Should we pull the current epoch from RPC or make it be a CLI argument?
    let current_epoch = 821;
    // TODO: Determine how this should be passed. The number of epochs to look back
    let look_back_period = 50;
    // TODO: Determine if this should be an argument
    let number_of_validator_delegations = 200;

    // Load existing steward config and overwrite parameters based on CLI args
    let mut steward_config = fetch_config(&rpc_client).await?;
    args.update_steward_config(&mut steward_config);

    let histories = ValidatorHistory::fetch_all(db_connection).await?;
    // Fetch the cluster history
    let cluster_history = ClusterHistory::fetch(db_connection).await?;
    let cluster_history_entries = ClusterHistoryEntry::fetch_all(db_connection).await?;
    // Convert cluster history to steward ClusterHistory
    let jito_cluster_history =
        cluster_history.convert_to_jito_cluster_history(cluster_history_entries);

    // For each validator, fetch their entries and score them
    let batch_size = 10;
    let futures: Vec<_> = histories
        .into_iter()
        .map(|x| {
            score_validator(
                db_connection,
                x,
                &jito_cluster_history,
                &steward_config,
                current_epoch,
            )
        })
        .collect();
    let results: Vec<_> = futures::stream::iter(futures)
        .buffer_unordered(batch_size)
        .collect()
        .await;
    let mut results: Vec<(String, f64)> = results.into_iter().filter_map(Result::ok).collect();
    // Sort the validator's by score
    results.sort_by(|a: &(String, f64), b| b.1.total_cmp(&a.1));

    // Take the top Y validators, fetch their epoch rewards and active stake
    let top_validators: Vec<String> = results
        .into_iter()
        .take(number_of_validator_delegations)
        .map(|x| x.0)
        .collect();
    let rewards = EpochRewards::fetch_for_validators_and_epochs(
        db_connection,
        &top_validators,
        (current_epoch - look_back_period).into(),
        current_epoch.into(),
    )
    .await?;
    // group the rewards by validator
    let mut validator_rewards: HashMap<String, Vec<EpochRewards>> = HashMap::new();
    for reward in rewards {
        validator_rewards
            .entry(reward.vote_pubkey.clone())
            .or_insert_with(Vec::new)
            .push(reward);
    }

    // Convert HashMap to Vec and sort each inner Vec by epoch
    let mut result: Vec<Vec<EpochRewards>> = validator_rewards.into_values().collect();
    for inner_vec in &mut result {
        inner_vec.sort_by_key(|reward| reward.epoch);
    }
    // Simulate 1 SOL being actively staked to each validator. For each epoch, the
    // active_stake input for the next epoch should increase by the proportional rewards
    // received.
    let lamports_after_staking: u64 = result
        .into_iter()
        .map(|x| {
            x.into_iter()
                .fold(LAMPORTS_PER_SOL, |current_active_stake, epoch_rewards| {
                    epoch_rewards.stake_after_epoch(current_active_stake)
                })
        })
        .sum();

    // Average the rate of return across all validators in the set.
    let total_starting_lamports = LAMPORTS_PER_SOL
        .checked_mul(number_of_validator_delegations as u64)
        .ok_or(CliError::ArithmeticError)?;

    let rate_of_return: f64 = (lamports_after_staking - total_starting_lamports)
        .to_f64()
        .ok_or(CliError::ArithmeticError)?
        / total_starting_lamports
            .to_f64()
            .ok_or(CliError::ArithmeticError)?;

    // Extrapolate to yearly for APY
    // Estimates epochs are 2 days (432_000 slots per epoch, 400ms per slot)
    let look_back_period_in_days =
        look_back_period.to_f64().ok_or(CliError::ArithmeticError)? * 2.0;
    assert!(look_back_period_in_days < DAYS_PER_YEAR);
    let apy = calculate_apy(rate_of_return, look_back_period_in_days, DAYS_PER_YEAR);
    info!("apy: {}", apy);

    Ok(())
}

pub async fn score_validator(
    db_connection: &Pool<Postgres>,
    validator_history: ValidatorHistory,
    jito_cluster_history: &JitoClusterHistory,
    steward_config: &Config,
    current_epoch: u16,
) -> Result<(String, f64), CliError> {
    let mut entries =
        ValidatorHistoryEntry::fetch_by_validator(db_connection, &validator_history.vote_account)
            .await?;
    let vote_account = validator_history.vote_account.clone();
    // Convert DB structures into on-chain structures
    let jito_validator_history = validator_history.convert_to_jito_validator_history(&mut entries);
    // Score the validator
    let score_result = validator_score(
        &jito_validator_history,
        jito_cluster_history,
        &steward_config,
        current_epoch,
        TVC_ACTIVATION_EPOCH,
    );
    match score_result {
        Ok(score) => Ok((vote_account, score.score)),
        Err(_) => {
            error!(
                "Erroring scoring validator {}",
                jito_validator_history.vote_account
            );
            Ok((vote_account, 0.0))
        }
    }
}

fn calculate_apy(r: f64, t: f64, n: f64) -> f64 {
    // APY = (1 + r)^(n/t) - 1
    (1.0 + r).powf(n / t) - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apy_calculation() {
        let r = 0.02; // 2% return
        let t = 2.0;  // 2-day period
        let n = 365.0; // Days in a year
        let apy = calculate_apy(r, t, n);
        assert!((apy - 36.113).abs() < 0.001, "APY calculation is incorrect");
    }
}