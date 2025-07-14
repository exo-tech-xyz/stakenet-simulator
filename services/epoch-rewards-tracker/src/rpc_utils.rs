use solana_client::{nonblocking::rpc_client::RpcClient, rpc_response::RpcInflationReward};
use solana_sdk::pubkey::Pubkey;

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
