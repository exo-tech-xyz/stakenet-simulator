use anyhow::{Ok, Result};
use clap::Parser;
use jito_steward::{constants::TVC_ACTIVATION_EPOCH, score::validator_score};
use sqlx::{Pool, Postgres};
use stakenet_simulator_db::{
    validator_history::ValidatorHistory, validator_history_entry::ValidatorHistoryEntry,
};

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
    // TODO: Should we pull the current epoch from RPC or make it be a CLI argument?
    let current_epoch = 817;
    let histories = ValidatorHistory::fetch_all(db_connection).await?;
    // TODO: Fetch the cluster history
    // TODO: Convert cluster history to steward ClusterHistory

    // TODO: Convert the args to Steward Config structure
    // For each validator, fetch their entries and score them
    for validator_history in histories {
        let mut entries = ValidatorHistoryEntry::fetch_by_validator(
            db_connection,
            &validator_history.vote_account,
        )
        .await?;
        // Convert DB structures into on-chain structures
        let jito_validator_history =
            validator_history.convert_to_jito_validator_history(&mut entries);
        // TODO: Score the validator
        // let score = validator_score(&jito_validator_history, cluster, config, current_epoch, TVC_ACTIVATION_EPOCH);
    }
    // TODO: Sort the validator's by score
    // TODO: Take the top Y validators, fetch their epoch rewards and active stake
    // TODO: Calculate the estimated combined APY if stake was evenly distributed across all the validators
    Ok(())
}
