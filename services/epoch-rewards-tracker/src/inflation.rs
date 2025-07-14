// TODO: For each validator load a stake account that has a long history

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey;

use crate::EpochRewardsTrackerError;

pub async fn get_inflation_rewards(rpc_client: &RpcClient) -> Result<(), EpochRewardsTrackerError> {
    let addresses = vec![pubkey!("2KxnNM2TEtUWYvsxhFk4qn3ix5CBohaXFVAzhn8iMuCS")];
    let rewards = crate::rpc_utils::get_inflation_rewards(rpc_client, &addresses, 801).await?;

    Ok(())
}

pub fn calculate_total_inflation_rewards(
    total_active_stake: u64,
    stake_amount: u64,
    commission: Option<u8>,
    inflation_rewards: u64,
) -> u64 {
    let total_rewards_for_stake_account = if let Some(commission) = commission {
        inflation_rewards
            .checked_mul(100)
            .and_then(|x| x.checked_div(u64::from(commission)))
            .unwrap()
    } else {
        inflation_rewards
    };
    total_rewards_for_stake_account
        .checked_mul(total_active_stake)
        .and_then(|x| x.checked_div(stake_amount))
        .unwrap()
}
