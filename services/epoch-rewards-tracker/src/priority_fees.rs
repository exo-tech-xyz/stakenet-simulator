use anchor_lang::{
    prelude::{EpochSchedule, SlotHistory},
    solana_program::example_mocks::solana_sdk::sysvar::slot_history,
};
use futures::future::join_all;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::reward_type::RewardType;
use sqlx::{Pool, Postgres};
use stakenet_simulator_db::epoch_priority_fees::EpochPriorityFees;
use tracing::{error, info};

use crate::{
    EpochRewardsTrackerError,
    rpc_utils::{RpcUtilsError, get_block},
};

pub async fn gather_priority_fee_data_for_epoch(
    db_connection: &Pool<Postgres>,
    rpc_client: &RpcClient,
    epoch: u64,
    epoch_schedule: &EpochSchedule,
    slot_history: &SlotHistory,
) -> Result<(), EpochRewardsTrackerError> {
    let first_slot_of_epoch = epoch_schedule.get_first_slot_in_epoch(epoch);
    // Fetch the leader schedule for the epoch
    info!("Begin: get_leader_schedule");
    let leader_schedule = rpc_client
        .get_leader_schedule(Some(first_slot_of_epoch))
        .await?
        .ok_or(EpochRewardsTrackerError::MissingLeaderSchedule(epoch))?;
    info!("Fin: get_leader_schedule");

    for (identity, leader_slots) in leader_schedule.into_iter() {
        let chunk_size = 25;
        info!("fetching blocks for leader {}", identity);

        let mut results = vec![];
        for slots in leader_slots.chunks(chunk_size) {
            let futures = slots
                .iter()
                .map(|&slot| {
                    let absolute_slot = first_slot_of_epoch + slot as u64;
                    let slot_history = slot_history.clone(); // Clone if needed

                    async move {
                        let result = get_block(&rpc_client, absolute_slot, &slot_history).await;
                        (absolute_slot, result)
                    }
                })
                .collect::<Vec<_>>();

            let future_results = join_all(futures).await;
            results.extend(future_results);
        }

        // Parse the block rewards and add to sum
        let total_fees = results
            .into_iter()
            .map(|(slot, get_block_result)| match get_block_result {
                Ok(block) => block
                    .rewards
                    .unwrap()
                    .into_iter()
                    .filter(|r| r.reward_type == Some(RewardType::Fee))
                    .map(|r| r.lamports as u64)
                    .sum::<u64>(),
                Err(e) => match e {
                    RpcUtilsError::SkippedBlock => 0,
                    _ => {
                        error!("Error with block {}: {:>}", slot, e);
                        0
                    }
                },
            })
            .sum::<u64>();
        EpochPriorityFees::bulk_insert(
            &db_connection,
            vec![EpochPriorityFees::new(identity, epoch, total_fees)],
        )
        .await?;
    }

    Ok(())
}
