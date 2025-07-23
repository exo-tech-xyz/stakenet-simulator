use crate::EpochRewardsTrackerError;
use anchor_lang::{AccountDeserialize, Discriminator};
use solana_account_decoder_client_types::{UiAccountEncoding, UiDataSliceConfig};
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, RpcFilterType},
};
use solana_sdk::pubkey::Pubkey;
use sqlx::{Pool, Postgres};
use stakenet_simulator_db::{
    validator_history::ValidatorHistory, validator_history_entry::ValidatorHistoryEntry,
};
use tracing::info;
use validator_history::ValidatorHistory as JitoValidatorHistory;

pub async fn load_and_record_validator_history(
    db_connection: &Pool<Postgres>,
    rpc_client: &RpcClient,
    program_id: Pubkey,
) -> Result<(), EpochRewardsTrackerError> {
    let current_epoch_info = rpc_client.get_epoch_info().await?;
    let last_finalized_epoch = current_epoch_info.epoch as u16;

    let validator_history_pubkeys =
        load_all_validator_history_pubkeys(&rpc_client, program_id).await?;
    info!("Validator history pubkeys: {:?}", validator_history_pubkeys);

    // Load validator history from jito program
    for validator_history_pubkey in validator_history_pubkeys.into_iter() {
        info!("Fetching ValidatorHistory at {}", validator_history_pubkey);
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
            JitoValidatorHistory::try_deserialize(&mut account.data.as_slice()).unwrap();
        let vote_pubkey = validator_history.vote_account;
        let entries: Vec<ValidatorHistoryEntry> = validator_history
            .history
            .epoch_range(last_finalized_epoch - 512, last_finalized_epoch)
            .into_iter()
            .filter_map(|x| {
                // Handle case where entry is basically null
                if x?.epoch == u16::MAX {
                    return None;
                }
                Some(ValidatorHistoryEntry::new(vote_pubkey.to_string(), *x?))
            })
            .collect();
        info!("Inserting {} entries for {}", entries.len(), vote_pubkey);
        ValidatorHistoryEntry::bulk_insert(db_connection, entries).await?;
        info!("Inserting ValidatorHistory for {}", vote_pubkey);
        ValidatorHistory::bulk_insert(db_connection, vec![validator_history.into()]).await?;
    }
    Ok(())
}

pub async fn load_all_validator_history_pubkeys(
    rpc_client: &RpcClient,
    program_id: Pubkey,
) -> Result<Vec<Pubkey>, EpochRewardsTrackerError> {
    let discriminator_filter = RpcFilterType::Memcmp(Memcmp::new_base58_encoded(
        0,
        &JitoValidatorHistory::DISCRIMINATOR,
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
