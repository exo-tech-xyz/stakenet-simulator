use anyhow::{Ok, Result};
use clap::Parser;
use sqlx::{Pool, Postgres};

#[derive(Clone, Debug, Parser)]
pub struct BacktestArgs {
    #[arg(long, env, default_value = "1")]
    mev_commission_range: u16,
    #[arg(long, env, default_value = "1")]
    epoch_credits_range: u16,
    #[arg(long, env, default_value = "1")]
    commission_range: u16,
    #[arg(long, env, default_value = "1.0")]
    scoring_delinquency_threshold_ratio: f64,
    #[arg(long, env, default_value = "1.0")]
    instant_unstake_delinquency_threshold_ratio: f64,
    #[arg(long, env, default_value = "1")]
    mev_commission_bps_threshold: u16,
    #[arg(long, env, default_value = "1")]
    commission_threshold: u8,
    #[arg(long, env, default_value = "1")]
    historical_commission_threshold: u8,
    #[arg(long, env, default_value = "1")]
    num_delegation_validators: u32,
    #[arg(long, env, default_value = "1")]
    scoring_unstake_cap_bps: u32,
    #[arg(long, env, default_value = "1")]
    instant_unstake_cap_bps: u32,
    #[arg(long, env, default_value = "1")]
    stake_deposit_unstake_cap_bps: u32,
    #[arg(long, env, default_value = "1.0")]
    instant_unstake_epoch_progress: f64,
    #[arg(long, env, default_value = "1")]
    compute_score_slot_range: u64,
    #[arg(long, env, default_value = "1.0")]
    instant_unstake_inputs_epoch_progress: f64,
    #[arg(long, env, default_value = "1")]
    num_epochs_between_scoring: u64,
    #[arg(long, env, default_value = "1")]
    minimum_stake_lamports: u64,
    #[arg(long, env, default_value = "1")]
    minimum_voting_epochs: u64,
    #[arg(long, env, default_value = "1")]
    target_epoch: u64,
}

pub async fn handle_backtest(args: BacktestArgs, db_connection: &Pool<Postgres>) -> Result<()> {
    Ok(())
}
