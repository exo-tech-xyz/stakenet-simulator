use anyhow::{Ok, Result};
use clap::Parser;
use sqlx::{Pool, Postgres};
use stakenet_simulator_db::validator_history_entry::fetch_all_validator_history_entries;

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
    let history_entries = fetch_all_validator_history_entries(db_connection).await?;
    Ok(())
}
