use anyhow::Result;

use clap::{Parser, Subcommand};
use commands::backtest::*;

pub mod commands;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Backtest {
        #[arg(short, long, default_value = "1")]
        mev_commission_range: u16,
        #[arg(short, long, default_value = "1")]
        epoch_credits_range: u16,
        #[arg(short, long, default_value = "1")]
        commission_range: u16,
        #[arg(short, long, default_value = "1.0")]
        scoring_delinquency_threshold_ratio: f64,
        #[arg(short, long, default_value = "1.0")]
        instant_unstake_delinquency_threshold_ratio: f64,
        #[arg(short, long, default_value = "1")]
        mev_commission_bps_threshold: u16,
        #[arg(short, long, default_value = "1")]
        commission_threshold: u8,
        #[arg(short, long, default_value = "1")]
        historical_commission_threshold: u8,
        #[arg(short, long, default_value = "1")]
        num_delegation_validators: u32,
        #[arg(short, long, default_value = "1")]
        scoring_unstake_cap_bps: u32,
        #[arg(short, long, default_value = "1")]
        instant_unstake_cap_bps: u32,
        #[arg(short, long, default_value = "1")]
        stake_deposit_unstake_cap_bps: u32,
        #[arg(short, long, default_value = "1.0")]
        instant_unstake_epoch_progress: f64,
        #[arg(short, long, default_value = "1")]
        compute_score_slot_range: u64,
        #[arg(short, long, default_value = "1.0")]
        instant_unstake_inputs_epoch_progress: f64,
        #[arg(short, long, default_value = "1")]
        num_epochs_between_scoring: u64,
        #[arg(short, long, default_value = "1")]
        minimum_stake_lamports: u64,
        #[arg(short, long, default_value = "1")]
        minimum_voting_epochs: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Backtest {
            mev_commission_range,
            epoch_credits_range,
            commission_range,
            scoring_delinquency_threshold_ratio,
            instant_unstake_delinquency_threshold_ratio,
            mev_commission_bps_threshold,
            commission_threshold,
            historical_commission_threshold,
            num_delegation_validators,
            scoring_unstake_cap_bps,
            instant_unstake_cap_bps,
            stake_deposit_unstake_cap_bps,
            instant_unstake_epoch_progress,
            compute_score_slot_range,
            instant_unstake_inputs_epoch_progress,
            num_epochs_between_scoring,
            minimum_stake_lamports,
            minimum_voting_epochs,
        } => {
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
                mev_commission_range,
                epoch_credits_range,
                commission_range,
                scoring_delinquency_threshold_ratio,
                instant_unstake_delinquency_threshold_ratio,
                mev_commission_bps_threshold,
                commission_threshold,
                historical_commission_threshold,
                num_delegation_validators,
                scoring_unstake_cap_bps,
                instant_unstake_cap_bps,
                stake_deposit_unstake_cap_bps,
                instant_unstake_epoch_progress,
                compute_score_slot_range,
                instant_unstake_inputs_epoch_progress,
                num_epochs_between_scoring,
                minimum_stake_lamports,
                minimum_voting_epochs,
            );
            handle_backtest(BacktestArgs {
                mev_commission_range,
                epoch_credits_range,
                commission_range,
                scoring_delinquency_threshold_ratio,
                instant_unstake_delinquency_threshold_ratio,
                mev_commission_bps_threshold,
                commission_threshold,
                historical_commission_threshold,
                num_delegation_validators,
                scoring_unstake_cap_bps,
                instant_unstake_cap_bps,
                stake_deposit_unstake_cap_bps,
                instant_unstake_epoch_progress,
                compute_score_slot_range,
                instant_unstake_inputs_epoch_progress,
                num_epochs_between_scoring,
                minimum_stake_lamports,
                minimum_voting_epochs,
            })
            .await
        }
    }
}
