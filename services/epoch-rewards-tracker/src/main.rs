use std::{str::FromStr, sync::Arc};

use solana_client::{client_error::ClientError, nonblocking::rpc_client::RpcClient};
use solana_sdk::pubkey::{ParsePubkeyError, Pubkey};
use sqlx::{Error as SqlxError, postgres::PgPoolOptions};
use thiserror::Error;
use tracing::{Level, error, info};
use tracing_subscriber::EnvFilter;

use crate::{
    cluster_history::load_and_record_cluster_history,
    config::{Config, ConfigError},
    inflation::{
        gather_inflation_rewards, gather_total_inflation_rewards_per_epoch, get_inflation_rewards,
    },
    priority_fees::gather_priority_fee_data_for_epoch,
    rpc_utils::{RpcUtilsError, fetch_slot_history},
    stake_accounts::gather_stake_accounts,
    validator_history::load_and_record_validator_history,
};

mod cluster_history;
mod config;
mod inflation;
mod priority_fees;
mod rpc_utils;
mod stake_accounts;
mod validator_history;

#[derive(Debug, Error)]
pub enum EpochRewardsTrackerError {
    #[error("ConfigError: {0}")]
    ConfigError(#[from] ConfigError),

    #[error("Solana ClientError: {0}")]
    ClientError(#[from] ClientError),

    #[error("ValidatorHistoryNotFound: {0}")]
    ValidatorHistoryNotFound(Pubkey),

    #[error("ClusterHistoryNotFound: {0}")]
    ClusterHistoryNotFound(Pubkey),

    #[error("SqlxError: {0}")]
    SqlxError(#[from] SqlxError),

    #[error("ParsePubkeyError: {0}")]
    ParsePubkeyError(#[from] ParsePubkeyError),

    #[error("RpcUtilsError: {0}")]
    RpcUtilsError(#[from] RpcUtilsError),

    #[error("MissingLeaderSchedule for epoch: {0}")]
    MissingLeaderSchedule(u64),
}

#[tokio::main]
async fn main() -> Result<(), EpochRewardsTrackerError> {
    let level = std::env::var("RUST_LOG").unwrap_or(Level::INFO.to_string());
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::new(level))
        // this needs to be set to remove duplicated information in the log.
        .with_current_span(false)
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        // remove the name of the function from every log entry
        .with_target(false)
        .init();

    let config = Config::from_env()?;

    let db_conn_pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&config.db_connection_url)
            .await
            .unwrap(),
    );
    let program_id = Pubkey::from_str(&config.validator_history_program_id).unwrap();
    let rpc_client = RpcClient::new(config.rpc_url.clone());

    // load_and_record_validator_history(&db_conn_pool, config.rpc_url, program_id).await?;
    load_and_record_cluster_history(&db_conn_pool, &rpc_client).await?;
    // get_inflation_rewards(&db_conn_pool, &rpc_client).await?;
    // gather_stake_accounts(&db_conn_pool, &rpc_client).await?;
    // gather_inflation_rewards(&db_conn_pool, &rpc_client).await?;
    // gather_total_inflation_rewards_per_epoch(&db_conn_pool).await?;
    // let epoch_schedule = rpc_client.get_epoch_schedule().await?;
    // let slot_history = fetch_slot_history(&rpc_client).await?;
    // gather_priority_fee_data_for_epoch(
    //     &db_conn_pool,
    //     &rpc_client,
    //     811,
    //     &epoch_schedule,
    //     &slot_history,
    // )
    // .await?;
    Ok(())
}
