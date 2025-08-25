use crate::{commands::DAYS_PER_YEAR, error::CliError, utils::RebalancingCycle};
use num_traits::cast::ToPrimitive;
use sqlx::{Pool, Postgres, types::BigDecimal};
use stakenet_simulator_db::{
    active_stake_jito_sol::ActiveStakeJitoSol, inactive_stake_jito_sol::InactiveStakeJitoSol,
};

pub fn calculate_apy(r: f64, t: f64, n: f64) -> f64 {
    // APY = (1 + r)^(n/t) - 1
    (1.0 + r).powf(n / t) - 1.0
}

pub fn calculate_aggregated_apy(
    rebalancing_cycles: &[RebalancingCycle],
    total_lookback_period: u16,
) -> Result<f64, CliError> {
    if rebalancing_cycles.is_empty() {
        return Ok(0.0);
    }

    // Get the initial and final total stake amounts
    let initial_total_stake = rebalancing_cycles[0].starting_total_lamports;
    let final_total_stake = rebalancing_cycles
        .last()
        .ok_or(CliError::ArithmeticError)?
        .ending_total_lamports;

    if initial_total_stake == 0 {
        return Ok(0.0);
    }

    let overall_return_rate = (final_total_stake - initial_total_stake)
        .to_f64()
        .ok_or(CliError::ArithmeticError)?
        / initial_total_stake
            .to_f64()
            .ok_or(CliError::ArithmeticError)?;

    // Convert to APY
    let lookback_period_in_days = total_lookback_period
        .to_f64()
        .ok_or(CliError::ArithmeticError)?
        * 2.0; // Assuming 2 days per epoch

    if lookback_period_in_days >= DAYS_PER_YEAR {
        return Err(CliError::LookBackPeriodTooBig);
    }

    let apy = calculate_apy(overall_return_rate, lookback_period_in_days, DAYS_PER_YEAR);

    Ok(apy)
}

fn calculate_stake_utilization(
    total_active_balance: &BigDecimal,
    total_inactive_balance: &BigDecimal,
) -> Result<f64, CliError> {
    let total_stake = total_active_balance.clone() + total_inactive_balance.clone();

    if total_stake == BigDecimal::from(0) {
        return Ok(0.0);
    }

    let utilization_rate = total_active_balance
        .to_f64()
        .ok_or(CliError::ArithmeticError)?
        / total_stake.to_f64().ok_or(CliError::ArithmeticError)?;

    Ok(utilization_rate)
}

pub async fn calculate_stake_utilization_rate(
    db_connection: &Pool<Postgres>,
    lookback_period: u16,
    current_epoch: u16,
) -> Result<f64, CliError> {
    if lookback_period > current_epoch {
        return Err(CliError::LookBackPeriodTooBig);
    }

    let (active_stake_data, inactive_stake_data) = futures::join!(
        ActiveStakeJitoSol::fetch_balance_for_epoch_range(
            db_connection,
            current_epoch as u64,
            lookback_period as u64,
        ),
        InactiveStakeJitoSol::fetch_balance_for_epoch_range(
            db_connection,
            current_epoch as u64,
            lookback_period as u64,
        )
    );

    let active_stake_data = active_stake_data?;
    let inactive_stake_data = inactive_stake_data?;

    if active_stake_data.count != inactive_stake_data.count {
        return Err(CliError::RecordCountMismatch {
            active_count: active_stake_data.count,
            inactive_count: inactive_stake_data.count,
        });
    }

    calculate_stake_utilization(&active_stake_data.balance, &inactive_stake_data.balance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::types::BigDecimal;

    #[test]
    fn test_apy_calculation() {
        let r = 0.02; // 2% return
        let t = 2.0; // 2-day period
        let n = 365.0; // Days in a year
        let apy = calculate_apy(r, t, n);
        assert!((apy - 36.113).abs() < 0.001, "APY calculation is incorrect");
    }

    #[test]
    fn test_calculate_stake_utilization_rate_from_balances() {
        // INACTIVE BALANCE is 0
        let active_balance = BigDecimal::from(100);
        let inactive_balance = BigDecimal::from(0);
        let result = calculate_stake_utilization(&active_balance, &inactive_balance);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1.0);

        // ACTIVE BALANCE is 0
        let active_balance = BigDecimal::from(0);
        let inactive_balance = BigDecimal::from(100);
        let result = calculate_stake_utilization(&active_balance, &inactive_balance);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);

        // TOTAL BALANCE is 0
        let active_balance = BigDecimal::from(0);
        let inactive_balance = BigDecimal::from(0);
        let result = calculate_stake_utilization(&active_balance, &inactive_balance);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);

        let active_balance = BigDecimal::from(800);
        let inactive_balance = BigDecimal::from(200);
        let result = calculate_stake_utilization(&active_balance, &inactive_balance);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.8);
    }
}
