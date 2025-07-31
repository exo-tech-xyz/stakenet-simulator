use solana_client::client_error::ClientError;
use sqlx::Error as SqlxError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Solana ClientError: {0}")]
    ClientError(#[from] ClientError),

    #[error("SqlxError: {0}")]
    SqlxError(#[from] SqlxError),

    #[error("AnchorDeserializeError")]
    AnchorDeserializeError,

    #[error("ArithmeticError")]
    ArithmeticError,
}
