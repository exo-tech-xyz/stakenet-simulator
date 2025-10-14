use crate::dune::{
    ACTIVE_STAKE_DUNE_QUERY, StakeRow, execute_dune_query, fetch_dune_query,
    wait_for_query_execution,
};
use crate::errors::EpochRewardsTrackerError;
use sqlx::types::BigDecimal;
use sqlx::{Pool, Postgres};
use stakenet_simulator_db::active_stake_jito_sol::ActiveStakeJitoSol;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::info;

pub async fn fetch_active_stake(db: &Pool<Postgres>) -> Result<(), EpochRewardsTrackerError> {
    let execute_client = execute_dune_query(ACTIVE_STAKE_DUNE_QUERY)
        .await
        .map_err(|_| EpochRewardsTrackerError::DuneApiError)?;

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

    let records: Vec<ActiveStakeJitoSol> = epoch_balances
        .into_iter()
        .map(|(epoch, balances)| {
            let avg_balance = balances.iter().sum::<f64>() / balances.len() as f64;
            ActiveStakeJitoSol::new(
                epoch,
                BigDecimal::from_str(&avg_balance.to_string()).unwrap(),
            )
        })
        .collect();

    if records.is_empty() {
        info!("No records to process.");
    } else {
        info!("Processing records...");
        ActiveStakeJitoSol::bulk_insert(db, records).await?;
        info!("Processing complete. Records inserted/updated.");
    }

    Ok(())
}
