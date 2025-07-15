use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey;
use solana_sdk::{pubkey::Pubkey, stake::state::StakeStateV2};
use sqlx::{Pool, Postgres};
use tracing::info;

use crate::{rpc_utils::fetch_stake_accounts_for_validator, EpochRewardsTrackerError};


pub async fn gather_stake_accounts(
  db_connection: &Pool<Postgres>,
  rpc_client: &RpcClient,
) -> Result<(), EpochRewardsTrackerError> {
  let vote_pubkey = pubkey!("9QU2QSxhb24FUX3Tu2FpczXjpK3VYrvRudywSZaM29mF");
  let res = fetch_stake_accounts_for_validator(rpc_client, &vote_pubkey).await?;
  info!("Fetched {} stake accounts", res.len());
  // TODO: Filter to find 10 with the longest history
  let mut res: Vec<(Pubkey, StakeStateV2)> = res.into_iter().filter(|x| x.1.stake().is_some()).collect();
  res.sort_by(|a, b| a.1.stake().unwrap().delegation.activation_epoch.cmp(&b.1.stake().unwrap().delegation.activation_epoch));
  // Take the first 10 elements (or fewer if the vector has less than 10)
  res.truncate(10);

  info!("Stake accounts: {:?}", res);
  Ok(()) 
}