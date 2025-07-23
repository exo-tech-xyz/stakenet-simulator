use anchor_lang::{AccountDeserialize, Discriminator};
use jito_steward::Config;
use solana_account_decoder_client_types::UiAccountEncoding;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig}, rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType}};
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use tracing::info;

use crate::EpochRewardsTrackerError;

pub const JITO_SOL_STAKE_POOL_ADDRESS: &str = "Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb";
pub const STAKE_POOL_PROGRAM: Pubkey = pubkey!("SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy");
pub const STEWARD_PROGRAM: Pubkey = pubkey!("Stewardf95sJbmtcZsyagb2dg4Mo8eVQho8gpECvLx8");

pub async fn fetch_and_log_steward_config(rpc_client: &RpcClient) -> Result<(), EpochRewardsTrackerError> {
  let discriminator_filter: RpcFilterType =
  RpcFilterType::Memcmp(Memcmp::new_base58_encoded(0, &Config::DISCRIMINATOR));
let stake_pool_filter = RpcFilterType::Memcmp(Memcmp::new(
  8,
  MemcmpEncodedBytes::Base58(String::from(JITO_SOL_STAKE_POOL_ADDRESS)),
));
let config = RpcProgramAccountsConfig {
  filters: Some(vec![
      discriminator_filter,
      stake_pool_filter,
  ]),
  account_config: RpcAccountInfoConfig {
      encoding: Some(UiAccountEncoding::Base64Zstd),
      data_slice: None,
      commitment: None,
      min_context_slot: None,
  },
  with_context: None,
  sort_results: Some(true),
};
let accounts = rpc_client
  .get_program_accounts_with_config(&STEWARD_PROGRAM, config)
  .await?;

  info!("found {} config accounts", accounts.len());

  accounts.iter().for_each(|(pubkey, account)| {
    let mut data: &[u8] = &account.data;
    let config = Config::try_deserialize(&mut data).unwrap();
    info!("Config: {} | admin {}", pubkey, config.admin);
  });

  Ok(())
}