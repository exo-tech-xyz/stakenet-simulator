use sqlx::types::BigDecimal;

pub mod active_stake_jito_sol;
mod big_decimal_u64;
pub mod cluster_history;
pub mod cluster_history_entry;
pub mod epoch_priority_fees;
pub mod epoch_rewards;
pub mod error;
pub mod inactive_stake_jito_sol;
pub mod inflation_rewards;
mod macros;
pub mod stake_accounts;
pub mod validator_history;
pub mod validator_history_entry;
pub mod withdraw_and_deposit_sol;
pub mod withdraw_and_deposits_stake;

#[derive(Debug)]
pub struct EpochBalanceResponse {
    pub balance: BigDecimal,
    pub count: i64,
}
