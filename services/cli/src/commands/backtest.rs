use anyhow::{Ok, Result, anyhow};

pub struct BacktestArgs {
    pub mev_commission_range: u16,

    pub epoch_credits_range: u16,

    pub commission_range: u16,

    pub scoring_delinquency_threshold_ratio: f64,

    pub instant_unstake_delinquency_threshold_ratio: f64,

    pub mev_commission_bps_threshold: u16,

    pub commission_threshold: u8,

    pub historical_commission_threshold: u8,

    pub num_delegation_validators: u32,

    pub scoring_unstake_cap_bps: u32,

    pub instant_unstake_cap_bps: u32,

    pub stake_deposit_unstake_cap_bps: u32,

    pub instant_unstake_epoch_progress: f64,

    pub compute_score_slot_range: u64,

    pub instant_unstake_inputs_epoch_progress: f64,

    pub num_epochs_between_scoring: u64,

    pub minimum_stake_lamports: u64,

    pub minimum_voting_epochs: u64,
}

pub async fn handle_backtest(args: BacktestArgs) -> Result<()> {
    Ok(())
}
