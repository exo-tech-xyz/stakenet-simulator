use anyhow::{Ok, Result};
use clap::Parser;

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
}

pub async fn handle_backtest(args: BacktestArgs) -> Result<()> {
    println!(
        "Hello world 
                mev_commission_range: {:?} 
                epoch_credits_range: {:?} 
                commission_range: {:?} 
                scoring_delinquency_threshold_ratio: {:?} 
                instant_unstake_delinquency_threshold_ratio: {:?} 
                mev_commission_bps_threshold {:?}
                commission_threshold: {:?} 
                historical_commission_threshold: {:?} 
                num_delegation_validators: {:?} 
                scoring_unstake_cap_bps: {:?} 
                instant_unstake_cap_bps {:?}
                stake_deposit_unstake_cap_bps: {:?} 
                instant_unstake_epoch_progress: {:?} 
                compute_score_slot_range: {:?} 
                instant_unstake_inputs_epoch_progress: {:?} 
                num_epochs_between_scoring: {:?} 
                minimum_stake_lamports {:?}
                minimum_voting_epochs: {:?} 
                ",
        args.mev_commission_range,
        args.epoch_credits_range,
        args.commission_range,
        args.scoring_delinquency_threshold_ratio,
        args.instant_unstake_delinquency_threshold_ratio,
        args.mev_commission_bps_threshold,
        args.commission_threshold,
        args.historical_commission_threshold,
        args.num_delegation_validators,
        args.scoring_unstake_cap_bps,
        args.instant_unstake_cap_bps,
        args.stake_deposit_unstake_cap_bps,
        args.instant_unstake_epoch_progress,
        args.compute_score_slot_range,
        args.instant_unstake_inputs_epoch_progress,
        args.num_epochs_between_scoring,
        args.minimum_stake_lamports,
        args.minimum_voting_epochs,
    );
    Ok(())
}
