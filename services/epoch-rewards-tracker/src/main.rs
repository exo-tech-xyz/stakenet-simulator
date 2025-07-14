use std::{str::FromStr, sync::Arc};

use solana_client::client_error::ClientError;
use solana_sdk::pubkey::Pubkey;
use sqlx::{Error as SqlxError, postgres::PgPoolOptions};
use thiserror::Error;
use tracing::{Level, error};
use tracing_subscriber::EnvFilter;

use crate::{
    config::{Config, ConfigError},
    validator_history::load_and_record_validator_history,
};

mod config;
mod inflation;
mod rpc_utils;
mod validator_history;

#[derive(Debug, Error)]
pub enum EpochRewardsTrackerError {
    #[error("ConfigError: {0}")]
    ConfigError(#[from] ConfigError),

    #[error("Solana ClientError: {0}")]
    ClientError(#[from] ClientError),

    #[error("ValidatorHistoryNotFound: {0}")]
    ValidatorHistoryNotFound(Pubkey),

    #[error("SqlxError: {0}")]
    SqlxError(#[from] SqlxError),
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

    load_and_record_validator_history(&db_conn_pool, config.rpc_url, program_id).await?;

    Ok(())
}
