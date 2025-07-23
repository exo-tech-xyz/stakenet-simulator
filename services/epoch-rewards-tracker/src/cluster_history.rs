use crate::EpochRewardsTrackerError;
use anchor_lang::AccountDeserialize;
use solana_account_decoder_client_types::UiAccountEncoding;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcAccountInfoConfig};
use solana_sdk::pubkey::Pubkey;
use sqlx::{Pool, Postgres};
use stakenet_simulator_db::{
    cluster_history::ClusterHistory, cluster_history_entry::ClusterHistoryEntry,
};
use tracing::info;
use validator_history::ClusterHistory as JitoClusterHistory;

pub async fn load_and_record_cluster_history(
    db_connection: &Pool<Postgres>,
    rpc_client: &RpcClient,
) -> Result<(), EpochRewardsTrackerError> {
    let current_epoch_info = rpc_client.get_epoch_info().await?;
    let last_finalized_epoch = current_epoch_info.epoch as u16;
    let cluster_history_pubkey =
        Pubkey::find_program_address(&[JitoClusterHistory::SEED], &validator_history::ID).0;
    let response = rpc_client
        .get_account_with_config(
            &cluster_history_pubkey,
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
        .ok_or(EpochRewardsTrackerError::ClusterHistoryNotFound(
            cluster_history_pubkey,
        ))?;
    let cluster_history =
        JitoClusterHistory::try_deserialize(&mut account.data.as_slice()).unwrap();
    let entries: Vec<ClusterHistoryEntry> = cluster_history
        .history
        .epoch_range(last_finalized_epoch - 512, last_finalized_epoch)
        .into_iter()
        .filter_map(
            |maybe_cluster_history_entry| match maybe_cluster_history_entry {
                Some(cluster_history_entry) => {
                    if cluster_history_entry.epoch == u16::MAX {
                        return None;
                    }
                    Some(cluster_history_entry.to_owned().into())
                }
                None => None,
            },
        )
        .collect();
    info!("Inserting {} cluster history entries ", entries.len());
    ClusterHistoryEntry::bulk_insert(db_connection, entries).await?;
    info!("Upserting cluster history ");
    ClusterHistory::upsert(db_connection, cluster_history.into()).await?;

    Ok(())
}
