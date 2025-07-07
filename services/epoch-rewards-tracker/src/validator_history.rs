use crate::EpochRewardsTrackerError;
use anchor_lang::{AccountDeserialize, Discriminator};
use solana_account_decoder_client_types::{UiAccountEncoding, UiDataSliceConfig};
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, RpcFilterType},
};
use solana_sdk::pubkey::Pubkey;
use tracing::info;
use validator_history::ValidatorHistory;

pub async fn load_and_record_validator_history(
    rpc_url: String,
    program_id: Pubkey,
) -> Result<(), EpochRewardsTrackerError> {
    let rpc_client = RpcClient::new(rpc_url);
    let validator_history_pubkeys =
        load_all_validator_history_pubkeys(&rpc_client, program_id).await?;
    info!("Validator history pubkeys: {:?}", validator_history_pubkeys);

    // TODO: Load validator history from jito program
    for validator_history_pubkey in validator_history_pubkeys.into_iter() {
        let response = rpc_client
            .get_account_with_config(
                &validator_history_pubkey,
                RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64Zstd),
                    data_slice: None,
                    commitment: None,
                    min_context_slot: None,
                },
            )
            .await?;
        let account = response
            .value
            .ok_or(EpochRewardsTrackerError::ValidatorHistoryNotFound(
                validator_history_pubkey,
            ))?;
        let validator_history =
            ValidatorHistory::try_deserialize(&mut account.data.as_slice()).unwrap();
    }
    Ok(())
}

pub async fn load_all_validator_history_pubkeys(
    rpc_client: &RpcClient,
    program_id: Pubkey,
) -> Result<Vec<Pubkey>, EpochRewardsTrackerError> {
    let discriminator_filter = RpcFilterType::Memcmp(Memcmp::new_base58_encoded(
        0,
        &ValidatorHistory::DISCRIMINATOR,
    ));
    let config = RpcProgramAccountsConfig {
        filters: Some(vec![discriminator_filter]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64Zstd),
            data_slice: Some(UiDataSliceConfig {
                offset: 0,
                length: 0,
            }),
            commitment: None,
            min_context_slot: None,
        },
        with_context: None,
        sort_results: None,
    };
    let accounts = rpc_client
        .get_program_accounts_with_config(&program_id, config)
        .await?;

    Ok(accounts.into_iter().map(|(pubkey, _)| pubkey).collect())
}
