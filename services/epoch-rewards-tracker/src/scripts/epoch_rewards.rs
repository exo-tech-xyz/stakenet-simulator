use csv::Reader;
use futures::future::join_all;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use stakenet_simulator_db::epoch_rewards::EpochRewards;
use std::sync::Arc;
use std::{collections::HashSet, error::Error};
use tokio::sync::Semaphore;
use tracing::{info, warn};

#[derive(Debug, Deserialize, Clone)]
struct EpochReward {
    epoch: u64,
    #[serde(alias = "vote_account")]
    vote_accounts: String,
    #[serde(default, deserialize_with = "empty_f64")]
    inflation_commission_pct: f64,
    #[serde(default, deserialize_with = "empty_f64")]
    total_inflation_rewards: f64,
    #[serde(default, deserialize_with = "empty_u64")]
    block_rewards: u64,
}

fn empty_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt
        .and_then(|s| if s.trim().is_empty() { None } else { Some(s) })
        .map(|s| s.parse::<f64>().unwrap_or(0.0))
        .unwrap_or(0.0))
}

fn empty_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt
        .and_then(|s| if s.trim().is_empty() { None } else { Some(s) })
        .map(|s| s.parse::<u64>().unwrap_or(0))
        .unwrap_or(0))
}

#[derive(Debug, Deserialize)]
struct ValidatorsResponse {
    validators: Vec<ValidatorInfo>,
}

#[derive(Debug, Deserialize)]
struct ValidatorInfo {
    vote_account: String,
    mev_commission_bps: Option<u16>,
    mev_rewards: Option<u64>,
    priority_fee_commission_bps: Option<u16>,
    active_stake: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("epoch_rewards binary started...");

    info!("Reading CSV file...");
    let mut rdr = Reader::from_path("./data.csv")?;
    let mut rows: Vec<EpochReward> = Vec::new();
    for result in rdr.deserialize() {
        rows.push(result?);
    }
    info!("Loaded {} rows from CSV", rows.len());

    let db_conn_pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect("postgresql://postgres:postgres@127.0.0.1:54322/postgres")
            .await?,
    );
    info!("Connected to Postgres");

    let rows = Arc::new(rows);
    let epochs = rows.iter().map(|r| r.epoch).collect::<HashSet<u64>>();
    let client = Arc::new(reqwest::Client::new());
    let semaphore = Arc::new(Semaphore::new(20));

    let tasks: Vec<_> = epochs
        .into_iter()
        .enumerate()
        .map(|(i, epoch)| {
            let client = client.clone();
            let permit = semaphore.clone().acquire_owned();
            let rows = rows.clone();
            let db_conn_pool = db_conn_pool.clone();

            tokio::spawn(async move {
                let _permit = permit.await.ok();
                info!("Fetching validators for epoch {} (task #{})", epoch, i);

                let resp = match client
                    .post("https://kobe.mainnet.jito.network/api/v1/validators")
                    .json(&serde_json::json!({ "epoch": epoch }))
                    .send()
                    .await
                {
                    Ok(r) => match r.json::<ValidatorsResponse>().await {
                        Ok(r) => r,
                        Err(e) => {
                            warn!("Failed to parse response for epoch {}: {}", epoch, e);
                            return;
                        }
                    },
                    Err(e) => {
                        warn!("Request failed for epoch {}: {}", epoch, e);
                        return;
                    }
                };

                let mut merged = Vec::new();
                for reward in rows.iter().filter(|r| r.epoch == epoch) {
                    if let Some(v) = resp
                        .validators
                        .iter()
                        .find(|val| val.vote_account == reward.vote_accounts)
                    {
                        merged.push(EpochRewards {
                            id: format!("{}-{}", reward.vote_accounts, reward.epoch),
                            vote_pubkey: reward.vote_accounts.clone(),
                            epoch: reward.epoch,
                            inflation_commission_bps: (reward.inflation_commission_pct * 100.0)
                                as u16,
                            total_inflation_rewards: reward.total_inflation_rewards as u64,
                            mev_commission_bps: v.mev_commission_bps.unwrap_or(0),
                            total_mev_rewards: v.mev_rewards.unwrap_or(0),
                            priority_fee_commission_bps: v
                                .priority_fee_commission_bps
                                .unwrap_or(10_000),
                            total_priority_fee_rewards: reward.block_rewards,
                            active_stake: v.active_stake,
                        });
                    }
                }

                if !merged.is_empty() {
                    match EpochRewards::bulk_insert(&db_conn_pool, merged).await {
                        Ok(_) => info!("Inserted records for epoch {}", epoch),
                        Err(e) => warn!("Failed to insert epoch {}: {}", epoch, e),
                    }
                }
            })
        })
        .collect();

    join_all(tasks).await;

    info!("Finished processing all epochs");
    Ok(())
}
