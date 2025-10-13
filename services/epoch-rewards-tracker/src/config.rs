use serde::Deserialize;
use thiserror::Error;

// Custom error type for configuration issues
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Configuration struct
#[derive(Deserialize, Debug)]
pub struct Config {
    pub rpc_url: String,
    /// The Validator History program id
    pub validator_history_program_id: String,
    /// Full connection string for the postgres DB
    pub db_connection_url: String,
}
