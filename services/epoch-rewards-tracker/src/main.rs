use solana_client::client_error::ClientError;
use solana_sdk::pubkey::Pubkey;
use thiserror::Error;
use tracing::{Level, error};
use tracing_subscriber::EnvFilter;

use crate::config::{Config, ConfigError};

mod config;
mod validator_history;

#[derive(Debug, Error)]
pub enum EpochRewardsTrackerError {
    #[error("ConfigError: {0}")]
    ConfigError(#[from] ConfigError),

    #[error("Solana ClientError: {0}")]
    ClientError(#[from] ClientError),

    #[error("ValidatorHistoryNotFound: {0}")]
    ValidatorHistoryNotFound(Pubkey),
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

    Ok(())
}
