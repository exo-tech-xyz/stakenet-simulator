use crate::{
    dune::{
        DEPOSIT_STAKE_TRANSACTIONS_QUERY, DepositsStakeRow, WITHDRAW_STAKE_TRANSACTIONS_QUERY,
        WithdrawStakeRow, execute_dune_query, fetch_dune_query, wait_for_query_execution,
    },
    errors::EpochRewardsTrackerError,
};
use num_traits::FromPrimitive;
use sqlx::{Pool, Postgres, types::BigDecimal};
use stakenet_simulator_db::withdraw_and_deposits_stake::WithdrawsAndDepositStakes;
use std::collections::HashMap;
use tracing::info;

pub async fn withdraw_and_deposits(db: &Pool<Postgres>) -> Result<(), EpochRewardsTrackerError> {
    let execute_client_deposit = execute_dune_query(DEPOSIT_STAKE_TRANSACTIONS_QUERY)
        .await
        .map_err(|_| EpochRewardsTrackerError::DuneApiError)?;
    wait_for_query_execution(&execute_client_deposit.execution_id).await?;
    let deposit_rows: Vec<DepositsStakeRow> =
        fetch_dune_query::<DepositsStakeRow>(execute_client_deposit.execution_id)
            .await
            .map_err(|_| EpochRewardsTrackerError::DuneApiError)?;
    info!("Deposit Rows: {}", deposit_rows.len());
    let execute_client_withdraw = execute_dune_query(WITHDRAW_STAKE_TRANSACTIONS_QUERY)
        .await
        .map_err(|_| EpochRewardsTrackerError::DuneApiError)?;
    wait_for_query_execution(&execute_client_withdraw.execution_id).await?;
    let withdraw_rows: Vec<WithdrawStakeRow> =
        fetch_dune_query::<WithdrawStakeRow>(execute_client_withdraw.execution_id)
            .await
            .map_err(|_| EpochRewardsTrackerError::DuneApiError)?;
    info!("Withdraws Rows: {}", withdraw_rows.len());

    let mut combined: HashMap<(u64, String), (f64, f64)> = HashMap::new();

    for row in deposit_rows {
        combined
            .entry((row.epoch, row.validator.clone()))
            .and_modify(|(deposit, _)| *deposit += row.deposit_stake)
            .or_insert((row.deposit_stake, 0.0));
    }

    for row in withdraw_rows {
        combined
            .entry((row.epoch, row.validator.clone()))
            .and_modify(|(_, withdraw)| *withdraw += row.withdraw_stake)
            .or_insert((0.0, row.withdraw_stake));
    }

    let mut merged: Vec<WithdrawsAndDepositStakes> = Vec::new();
    for ((epoch, validator), (deposit, withdraw)) in combined {
        merged.push(WithdrawsAndDepositStakes::new(
            epoch,
            validator,
            BigDecimal::from_f64(withdraw).unwrap_or_else(|| BigDecimal::from(0)),
            BigDecimal::from_f64(deposit).unwrap_or_else(|| BigDecimal::from(0)),
        ));
    }

    if merged.is_empty() {
        info!("No records to process.");
    } else {
        info!("Processing {} records...", merged.len());
        WithdrawsAndDepositStakes::bulk_insert(db, merged).await?;
        info!("Processing complete. Records inserted/updated.");
    }

    Ok(())
}
