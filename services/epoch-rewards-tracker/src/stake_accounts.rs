use crate::{EpochRewardsTrackerError, rpc_utils::fetch_stake_accounts_for_validator};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, stake::state::StakeStateV2};
use sqlx::{Pool, Postgres};
use stakenet_simulator_db::stake_accounts::StakeAccount;
use stakenet_simulator_db::validator_history_entry::ValidatorHistoryEntry;
use std::str::FromStr;
use tracing::info;

pub async fn gather_stake_accounts(
    db_connection: &Pool<Postgres>,
    rpc_client: &RpcClient,
) -> Result<(), EpochRewardsTrackerError> {
    let vote_keys = ValidatorHistoryEntry::get_all_vote_pubkeys(db_connection).await?;

    info!("Fetched {} vote keys", vote_keys.len());
    for vote_key in vote_keys {
        let vote_pubkey = Pubkey::from_str(&vote_key)?;
        let res = fetch_stake_accounts_for_validator(rpc_client, &vote_pubkey).await?;
        info!(
            "Fetched {} stake accounts for vote account {}",
            res.len(),
            vote_key
        );
        // Find [at most] 10 with the longest history. First filter to make sure Stake structure
        //  exists on the account and at least 0.1 SOL is delegated
        let mut res: Vec<(Pubkey, StakeStateV2)> = res
            .into_iter()
            .filter(|x| {
                x.1.stake().is_some()
                    && x.1.delegation().is_some()
                    && x.1.delegation().unwrap().stake > 100_000_000
            })
            .collect();
        res.sort_by(|a, b: &(Pubkey, StakeStateV2)| {
            a.1.stake()
                .unwrap()
                .delegation
                .activation_epoch
                .cmp(&b.1.stake().unwrap().delegation.activation_epoch)
        });
        // Take the first 10 elements (or fewer if the vector has less than 10)
        res.truncate(10);
        let records: Vec<StakeAccount> = res.into_iter().map(|x| x.into()).collect();
        StakeAccount::bulk_insert(db_connection, records).await?;
    }

    Ok(())
}
