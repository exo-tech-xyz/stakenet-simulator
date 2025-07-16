use serde::Deserialize;
use thiserror::Error;

// Custom error type for configuration issues
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Environment variable {0} is missing")]
    MissingEnvVar(String),
    #[error("Environment variable {0} has invalid format: {1}")]
    InvalidEnvVar(String, String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Configuration struct
#[derive(Deserialize, Debug)]
pub struct Config {
    /// Full connection string for the postgres DB
    pub db_connection_url: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let config = envy::from_env::<Config>().map_err(|e| {
            let error_str = e.to_string();
            if error_str.contains("not found") {
                let var_name = error_str
                    .split_whitespace()
                    .nth(2)
                    .unwrap_or("unknown")
                    .to_string();
                ConfigError::MissingEnvVar(var_name)
            } else {
                ConfigError::InvalidEnvVar("unknown".to_string(), error_str)
            }
        })?;

        Ok(config)
    }
}
