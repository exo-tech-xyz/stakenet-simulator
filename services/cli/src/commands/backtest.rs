use crate::utils::{
    calculate_aggregated_apy, calculate_stake_utilization_rate, rebalancing_simulation,
};
use crate::{error::CliError, modify_config_parameter_from_args, steward_utils::fetch_config};
use clap::Parser;
use jito_steward::Config;
use solana_client::nonblocking::rpc_client::RpcClient;
use sqlx::{Pool, Postgres};
use tracing::info;

pub const DAYS_PER_YEAR: f64 = 365.0;

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
    #[arg(long, env, default_value = "10")]
    steward_cycle_rate: u16,
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
    let current_epoch: u16 = 821;
    // TODO: Determine how this should be passed. The number of epochs to look back
    let look_back_period = 50;
    // TODO: Determine if this should be an argument
    let number_of_validator_delegations = 200;

    // Load existing steward config and overwrite parameters based on CLI args
    let mut steward_config = fetch_config(&rpc_client).await?;
    args.update_steward_config(&mut steward_config);

    let simulation_start_epoch = current_epoch.saturating_sub(look_back_period);
    let simulation_end_epoch = std::cmp::min(current_epoch, current_epoch);

    let rebalancing_cycles = rebalancing_simulation(
        db_connection,
        &steward_config,
        simulation_start_epoch,
        simulation_end_epoch,
        args.steward_cycle_rate,
        number_of_validator_delegations,
        steward_config.parameters.instant_unstake_cap_bps,
    )
    .await?;

    let aggregated_apy = calculate_aggregated_apy(&rebalancing_cycles, look_back_period)?;

    let stake_utilization_ratio =
        calculate_stake_utilization_rate(db_connection, look_back_period, current_epoch).await?;

    let final_apy = aggregated_apy * stake_utilization_ratio;

    info!("Rebalancing cycles completed: {}", rebalancing_cycles.len());
    info!("Raw aggregated APY: {:.4}%", aggregated_apy * 100.0);
    info!("Stake utilization ratio: {:.4}", stake_utilization_ratio);
    info!("Final adjusted APY: {:.4}%", final_apy * 100.0);

    Ok(())
}
