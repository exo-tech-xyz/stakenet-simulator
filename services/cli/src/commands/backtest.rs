use anyhow::{Ok, Result};
use clap::Parser;
use sqlx::{Pool, Postgres};
use stakenet_simulator_db::validator_history_entry::ValidatorHistoryEntry;

#[derive(Clone, Debug, Parser)]
pub struct BacktestArgs {
    #[arg(long, env, default_value = "1")]
    pub mev_commission_range: u16,
    #[arg(long, env, default_value = "1")]
    pub epoch_credits_range: u16,
    #[arg(long, env, default_value = "1")]
    pub commission_range: u16,
    #[arg(long, env, default_value = "1.0")]
    pub scoring_delinquency_threshold_ratio: f64,
    #[arg(long, env, default_value = "1.0")]
    pub instant_unstake_delinquency_threshold_ratio: f64,
    #[arg(long, env, default_value = "1")]
    pub mev_commission_bps_threshold: u16,
    #[arg(long, env, default_value = "1")]
    pub commission_threshold: u8,
    #[arg(long, env, default_value = "1")]
    pub historical_commission_threshold: u8,
    #[arg(long, env, default_value = "1")]
    pub num_delegation_validators: u32,
    #[arg(long, env, default_value = "1")]
    pub scoring_unstake_cap_bps: u32,
    #[arg(long, env, default_value = "1")]
    pub instant_unstake_cap_bps: u32,
    #[arg(long, env, default_value = "1")]
    pub stake_deposit_unstake_cap_bps: u32,
    #[arg(long, env, default_value = "1.0")]
    pub instant_unstake_epoch_progress: f64,
    #[arg(long, env, default_value = "1")]
    pub compute_score_slot_range: u64,
    #[arg(long, env, default_value = "1.0")]
    pub instant_unstake_inputs_epoch_progress: f64,
    #[arg(long, env, default_value = "1")]
    pub num_epochs_between_scoring: u64,
    #[arg(long, env, default_value = "1")]
    pub minimum_stake_lamports: u64,
    #[arg(long, env, default_value = "1")]
    pub minimum_voting_epochs: u64,
    #[arg(long, env, default_value = "1")]
    target_epoch: u64,
}

pub async fn handle_backtest(args: BacktestArgs, db_connection: &Pool<Postgres>) -> Result<()> {
    // TODO (nice to have): Modify this fetch to only get the last X epochs worth of entries from
    // each validator. X should be the longest range for a scoring metric.
    let history_entries =
        ValidatorHistoryEntry::fetch_all_validator_history_entries(db_connection).await?;
    // TODO: Load the cluster history entries mapped by epoch

    // TODO: map entries by valdiator, then epoch. Besure to use the u16 epoch representation
    // TODO: For each validator run the valdiator_score algorithm and sort validators vote keys by
    //  their score. Be sure to use modified Config parameters.
    // TODO: Take the top Y validators, fetch their epoch rewards and active stake
    // TODO: Calculate the estimated combined APY if stake was evenly distributed across all the validators
    Ok(())
}
