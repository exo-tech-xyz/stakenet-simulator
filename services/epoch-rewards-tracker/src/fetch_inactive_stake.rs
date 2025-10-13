use crate::{
    dune::{
        INACTIVE_STAKE_DUNE_QUERY, StakeRow, execute_dune_query, fetch_dune_query,
        wait_for_query_execution,
    },
    errors::EpochRewardsTrackerError,
};
use sqlx::{Pool, Postgres, types::BigDecimal};
use stakenet_simulator_db::inactive_stake_jito_sol::InactiveStakeJitoSol;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::info;

pub async fn fetch_inactive_stake(db: &Pool<Postgres>) -> Result<(), EpochRewardsTrackerError> {
    let execute_client = execute_dune_query(INACTIVE_STAKE_DUNE_QUERY)
        .await
        .map_err(|_| EpochRewardsTrackerError::DuneApiError)?;

    if execute_client.execution_id.is_empty() {
        return Err(EpochRewardsTrackerError::EmptyExecutionId);
    }

    wait_for_query_execution(&execute_client.execution_id).await?;

    let results: Vec<StakeRow> = fetch_dune_query(execute_client.execution_id)
        .await
        .map_err(|_| EpochRewardsTrackerError::DuneApiError)?;

    let mut epoch_balances: HashMap<u64, Vec<f64>> = HashMap::new();

    for row in results {
        epoch_balances
            .entry(row.approx_epoch)
            .or_default()
            .push(row.total_sol_balance);
    }

    let records: Vec<InactiveStakeJitoSol> = epoch_balances
        .into_iter()
        .map(|(epoch, balances)| {
            let avg_balance = balances.iter().sum::<f64>() / balances.len() as f64;
            InactiveStakeJitoSol::new(
                epoch,
                BigDecimal::from_str(&avg_balance.to_string()).unwrap(),
            )
        })
        .collect();

    if records.is_empty() {
        info!("No records to process.");
    } else {
        info!("Processing records...");
        InactiveStakeJitoSol::bulk_insert(db, records).await?;
        info!("Processing complete. Records inserted/updated.");
    }

    Ok(())
}
