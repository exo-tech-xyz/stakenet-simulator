use anchor_lang::AccountDeserialize;
use jito_steward::Config;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;

use crate::error::CliError;

pub const STEWARD_CONFIG_PUBKEY: Pubkey = pubkey!("jitoVjT9jRUyeXHzvCwzPgHj7yWNRhLcUoXtes4wtjv");

pub async fn fetch_config(rpc_client: &RpcClient) -> Result<Config, CliError> {
    let account = rpc_client.get_account(&STEWARD_CONFIG_PUBKEY).await?;
    let mut data: &[u8] = &account.data;
    Ok(Config::try_deserialize(&mut data).map_err(|_| CliError::AnchorDeserializeError)?)
}
