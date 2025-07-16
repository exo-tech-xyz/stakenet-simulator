use solana_account_decoder_client_types::UiAccountEncoding;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
    rpc_response::RpcInflationReward,
};
use solana_sdk::{
    pubkey::Pubkey,
    stake::{self, state::StakeStateV2},
};

use crate::EpochRewardsTrackerError;

pub async fn get_inflation_rewards(
    rpc_client: &RpcClient,
    stake_accounts: &[Pubkey],
    epoch: u64,
) -> Result<Vec<Option<RpcInflationReward>>, EpochRewardsTrackerError> {
    let res = rpc_client
        .get_inflation_reward(stake_accounts, Some(epoch))
        .await?;
    Ok(res)
}

pub async fn fetch_stake_accounts_for_validator(
    client: &RpcClient,
    vote_pubkey: &Pubkey,
) -> Result<Vec<(Pubkey, StakeStateV2)>, EpochRewardsTrackerError> {
    let discriminator_filter =
        RpcFilterType::Memcmp(Memcmp::new_base58_encoded(0, &2u32.to_le_bytes()));
    let vote_pubkey_filter = RpcFilterType::Memcmp(Memcmp::new(
        4 + 120, // u32 enum + size of Meta
        MemcmpEncodedBytes::Base58(vote_pubkey.to_string()),
    ));
    let blank_decativation_epoch = RpcFilterType::Memcmp(Memcmp::new_base58_encoded(
        4 + 120 + 48, // u32 enum + size of Meta + offset in Satke
        &u64::MAX.to_le_bytes(),
    ));
    let config = RpcProgramAccountsConfig {
        filters: Some(vec![
            discriminator_filter,
            vote_pubkey_filter,
            blank_decativation_epoch,
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
    let accounts = client
        .get_program_accounts_with_config(&stake::program::ID, config)
        .await?;

    Ok(accounts
        .into_iter()
        .map(|(pubkey, account)| {
            let mut data: &[u8] = &account.data;
            let bond = <StakeStateV2 as borsh::BorshDeserialize>::deserialize(&mut data).unwrap();
            (pubkey, bond)
        })
        .collect())
}
