use thiserror::Error;

#[derive(Debug, Error)]
pub enum StakenetSimulatorDbError {
    #[error("Error decodign column {0}")]
    DecodeError(String),
}
