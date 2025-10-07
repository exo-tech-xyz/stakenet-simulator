use crate::{
    dune::{
        WITHDRAW_DEPOSIT_SOL_QUERY, WithdrawDepositSol, execute_dune_query, fetch_dune_query,
        wait_for_query_execution,
    },
    errors::EpochRewardsTrackerError,
};
use num_traits::FromPrimitive;
use sqlx::{Pool, Postgres, types::BigDecimal};
use stakenet_simulator_db::withdraw_and_deposit_sol::WithdrawAndDepositSol;
use tracing::info;

pub async fn withdraw_and_deposit_sol(db: &Pool<Postgres>) -> Result<(), EpochRewardsTrackerError> {
    let execute_client = execute_dune_query(WITHDRAW_DEPOSIT_SOL_QUERY)
        .await
        .map_err(|_| EpochRewardsTrackerError::DuneApiError)?;

    if execute_client.execution_id.is_empty() {
        return Err(EpochRewardsTrackerError::EmptyExecutionId);
    }

    wait_for_query_execution(&execute_client.execution_id).await?;

    let results: Vec<WithdrawDepositSol> = fetch_dune_query(execute_client.execution_id)
        .await
        .map_err(|_| EpochRewardsTrackerError::DuneApiError)?;

    let mut withdraw_and_deposit_sols: Vec<WithdrawAndDepositSol> =
        Vec::with_capacity(results.len());

    for r in results {
        let withdraw_bd =
            BigDecimal::from_f64(r.withdraw_sol).unwrap_or_else(|| BigDecimal::from(0));
        let deposit_bd = BigDecimal::from_f64(r.deposit_sol).unwrap_or_else(|| BigDecimal::from(0));

        withdraw_and_deposit_sols.push(WithdrawAndDepositSol::new(
            r.epoch,
            withdraw_bd,
            deposit_bd,
        ));
    }

    if withdraw_and_deposit_sols.is_empty() {
        info!("No records to process.");
    } else {
        info!("Processing {} records...", withdraw_and_deposit_sols.len());
        WithdrawAndDepositSol::bulk_insert(db, withdraw_and_deposit_sols).await?;
        info!("Processing complete. Records inserted/updated.");
    }

    Ok(())
}
