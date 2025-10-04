use crate::errors::EpochRewardsTrackerError;
use dotenvy::dotenv;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, de::DeserializeOwned};
use std::collections::HashSet;
use std::env;
use tokio::time::{Duration, sleep};
use tracing::{info, warn};

#[derive(Debug, Deserialize, PartialEq)]
pub struct StakeRow {
    pub day: String,
    pub total_sol_balance: f64,
    pub approx_epoch: u64,
}

#[derive(Debug, Deserialize)]
pub struct WithdrawStakeRow {
    pub epoch: u64,
    pub validator: String,
    pub withdraw_stake: f64,
}

#[derive(Debug, Deserialize)]
pub struct DepositsStakeRow {
    pub epoch: u64,
    pub validator: String,
    pub deposit_stake: f64,
}

#[derive(Debug, Deserialize)]
pub struct WithdrawDepositSol {
    pub epoch: u64,
    pub withdraw_sol: f64,
    pub deposit_sol: f64,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteResponse {
    pub execution_id: String,
}

#[derive(Debug, Deserialize)]
struct ResultData<T> {
    rows: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    result: ResultData<T>,
}

#[derive(Debug, Deserialize)]
pub struct ExecutionStatus {
    pub state: String,
    // "QUERY_STATE_PENDING" state
    pub queue_position: Option<u32>,
}

pub const INACTIVE_STAKE_DUNE_QUERY: u64 = 5571499;
pub const ACTIVE_STAKE_DUNE_QUERY: u64 = 5571504;
pub const DEPOSIT_STAKE_TRANSACTIONS_QUERY: u64 = 5759079;
pub const WITHDRAW_STAKE_TRANSACTIONS_QUERY: u64 = 5751846;
pub const WITHDRAW_DEPOSIT_SOL_QUERY: u64 = 5904420;

pub async fn execute_dune_query(
    query_id: u64,
) -> Result<ExecuteResponse, Box<dyn std::error::Error>> {
    dotenv().ok();

    let api_key = env::var("DUNE_API_KEY")?;
    let url = format!("https://api.dune.com/api/v1/query/{}/execute", query_id);

    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert("X-Dune-API-Key", HeaderValue::from_str(&api_key)?);
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    let response = client
        .post(&url)
        .headers(headers)
        .send()
        .await?
        .error_for_status()?
        .json::<ExecuteResponse>()
        .await?;

    Ok(response)
}

pub async fn fetch_dune_query<T>(execution_id: String) -> Result<Vec<T>, Box<dyn std::error::Error>>
where
    T: DeserializeOwned,
{
    dotenv().ok();
    let api_key = env::var("DUNE_API_KEY")?;
    let url = format!("https://api.dune.com/api/v1/execution/{execution_id}/results");
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert("X-Dune-API-Key", HeaderValue::from_str(&api_key)?);

    let response = client
        .get(&url)
        .headers(headers)
        .send()
        .await?
        .error_for_status()?
        .json::<ApiResponse<T>>()
        .await?;

    Ok(response.result.rows)
}

pub async fn fetch_dune_execution_status(
    execution_id: &str,
) -> Result<ExecutionStatus, Box<dyn std::error::Error>> {
    dotenv().ok();

    let api_key = env::var("DUNE_API_KEY")?;
    let url = format!(
        "https://api.dune.com/api/v1/execution/{}/status",
        execution_id
    );

    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert("X-Dune-API-Key", HeaderValue::from_str(&api_key)?);

    let response = client
        .get(&url)
        .headers(headers)
        .send()
        .await?
        .error_for_status()?
        .json::<ExecutionStatus>()
        .await?;

    Ok(response)
}

pub async fn wait_for_query_execution(execution_id: &str) -> Result<(), EpochRewardsTrackerError> {
    if execution_id.is_empty() {
        return Err(EpochRewardsTrackerError::EmptyExecutionId);
    }
    info!("Submitted Dune query. Execution ID: {}", execution_id);
    let mut seen_states = HashSet::new();
    loop {
        let status = fetch_dune_execution_status(execution_id)
            .await
            .map_err(|_| EpochRewardsTrackerError::DuneApiError)?;
        let state_str = status.state.as_str();

        // TODO: Have an enum for Dune states

        if !seen_states.contains(state_str) {
            match state_str {
                "QUERY_STATE_COMPLETED" => {
                    info!("Query execution completed!");
                    break;
                }
                "QUERY_STATE_PENDING" => {
                    info!(
                        "Query pending... Queue position: {:?}",
                        status.queue_position
                    );
                }
                "QUERY_STATE_EXECUTING" => {
                    info!("Query executing... hang tight.");
                }
                other => {
                    warn!("Unexpected query state: {}", other);
                }
            }
            seen_states.insert(state_str.to_string());
        }
        if state_str == "QUERY_STATE_COMPLETED" {
            break;
        }
        sleep(Duration::from_secs(10)).await;
    }
    Ok(())
}
