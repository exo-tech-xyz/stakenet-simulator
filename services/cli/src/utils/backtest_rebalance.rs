use crate::error::CliError;
use futures::StreamExt;
use jito_steward::{Config, constants::TVC_ACTIVATION_EPOCH, score::validator_score};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use sqlx::{Pool, Postgres};
use stakenet_simulator_db::{
    cluster_history::ClusterHistory, cluster_history_entry::ClusterHistoryEntry,
    epoch_rewards::EpochRewards, validator_history::ValidatorHistory,
    validator_history_entry::ValidatorHistoryEntry,
};
use std::collections::HashMap;
use tracing::{error, info};
use validator_history::ClusterHistory as JitoClusterHistory;

#[derive(Clone, Debug)]
pub struct RebalancingCycle {
    pub start_epoch: u16,
    pub end_epoch: u16,
    pub selected_validators: Vec<String>,
    pub starting_total_lamports: u64,
    pub ending_total_lamports: u64,
    pub starting_stake_per_validator: u64,
}

pub async fn rebalancing_simulation(
    db_connection: &Pool<Postgres>,
    steward_config: &Config,
    simulation_start_epoch: u16,
    simulation_end_epoch: u16,
    steward_cycle_rate: u16,
    number_of_validator_delegations: usize,
) -> Result<Vec<RebalancingCycle>, CliError> {
    let mut rebalancing_cycles = Vec::new();
    let mut current_cycle_start = simulation_start_epoch;

    let mut lamports_after_staking = LAMPORTS_PER_SOL
        .checked_mul(number_of_validator_delegations as u64)
        .ok_or(CliError::ArithmeticError)?;

    let histories = ValidatorHistory::fetch_all(db_connection).await?;
    let cluster_history = ClusterHistory::fetch(db_connection).await?;
    let cluster_history_entries = ClusterHistoryEntry::fetch_all(db_connection).await?;
    let jito_cluster_history =
        cluster_history.convert_to_jito_cluster_history(cluster_history_entries);

    // TODO: Add check to ensure that we don't go past the current epoch
    while current_cycle_start < simulation_end_epoch {
        let current_cycle_end = std::cmp::min(
            current_cycle_start + steward_cycle_rate,
            simulation_end_epoch,
        );

        let starting_stake_per_validator =
            lamports_after_staking / number_of_validator_delegations as u64;

        // TODO: Currently this is being calculated for each cycle, but we could optimize this as 
        // we know the epoch rewards. So, we can initally calculate the top validators in parallel for each cycle.
        // This could decrease the backtesting time by a lot.
        let top_validators = top_validators_for_epoch(
            db_connection,
            &histories,
            &jito_cluster_history,
            steward_config,
            current_cycle_start,
            number_of_validator_delegations,
        )
        .await?;

        let (cycle_ending_lamports, cycle_result) = simulate_returns(
            db_connection,
            &top_validators,
            current_cycle_start,
            current_cycle_end,
            lamports_after_staking,
            starting_stake_per_validator,
        )
        .await?;

        info!(
            "Cycle ending lamports: {:.3} SOL (was {:.3} SOL)",
            cycle_ending_lamports as f64 / LAMPORTS_PER_SOL as f64,
            lamports_after_staking as f64 / LAMPORTS_PER_SOL as f64
        );

        lamports_after_staking = cycle_ending_lamports;

        rebalancing_cycles.push(cycle_result);
        current_cycle_start = current_cycle_end;
    }

    Ok(rebalancing_cycles)
}

/// Scores all validators at `scoring_epoch` and returns the top `number_of_validators` vote 
/// account pubkeys
async fn top_validators_for_epoch(
    db_connection: &Pool<Postgres>,
    histories: &[ValidatorHistory],
    jito_cluster_history: &JitoClusterHistory,
    steward_config: &Config,
    scoring_epoch: u16,
    number_of_validators: usize,
) -> Result<Vec<String>, CliError> {
    info!("Scoring validators for epoch {}", scoring_epoch);

    let batch_size = 10;
    let futures: Vec<_> = histories
        .iter()
        .map(|validator_history| {
            score_validator(
                db_connection,
                validator_history.clone(),
                jito_cluster_history,
                steward_config,
                scoring_epoch,
            )
        })
        .collect();

    let results: Vec<_> = futures::stream::iter(futures)
        .buffer_unordered(batch_size)
        .collect()
        .await;

    let mut scored_validators: Vec<(String, f64)> =
        results.into_iter().filter_map(Result::ok).collect();

    scored_validators.sort_by(|a, b| b.1.total_cmp(&a.1));

    // TODO: Handle the zero score case too I guess.
    let top_validators: Vec<String> = scored_validators
        .into_iter()
        .take(number_of_validators)
        .map(|(vote_account, _score)| vote_account)
        .collect();

    Ok(top_validators)
}

pub async fn score_validator(
    db_connection: &Pool<Postgres>,
    validator_history: ValidatorHistory,
    jito_cluster_history: &JitoClusterHistory,
    steward_config: &Config,
    current_epoch: u16,
) -> Result<(String, f64), CliError> {
    let mut entries =
        ValidatorHistoryEntry::fetch_by_validator(db_connection, &validator_history.vote_account)
            .await?;
    let vote_account = validator_history.vote_account.clone();
    // Convert DB structures into on-chain structures
    let jito_validator_history = validator_history.convert_to_jito_validator_history(&mut entries);
    // Score the validator
    let score_result = validator_score(
        &jito_validator_history,
        jito_cluster_history,
        &steward_config,
        current_epoch,
        TVC_ACTIVATION_EPOCH,
    );
    match score_result {
        Ok(score) => Ok((vote_account, score.score)),
        Err(_) => {
            error!(
                "Erroring scoring validator {}",
                jito_validator_history.vote_account
            );
            Ok((vote_account, 0.0))
        }
    }
}

/// Simulates a Steward cycle where stake has been rebalanced and each validator starts with 
/// `starting_stake_per_validator`.
async fn simulate_returns(
    db_connection: &Pool<Postgres>,
    selected_validators: &[String],
    cycle_start_epoch: u16,
    cycle_end_epoch: u16,
    total_starting_lamports: u64,
    starting_stake_per_validator: u64,
) -> Result<(u64, RebalancingCycle), CliError> {
    info!(
        "Simulating returns for {} validators from epoch {} to {} with {:.3} SOL per validator",
        selected_validators.len(),
        cycle_start_epoch,
        cycle_end_epoch,
        starting_stake_per_validator as f64 / LAMPORTS_PER_SOL as f64
    );

    let rewards = EpochRewards::fetch_for_validators_and_epochs(
        db_connection,
        &selected_validators.to_vec(),
        cycle_start_epoch.into(),
        cycle_end_epoch.into(),
    )
    .await?;

    let mut validator_rewards: HashMap<String, Vec<EpochRewards>> = HashMap::new();
    for reward in rewards {
        validator_rewards
            .entry(reward.vote_pubkey.clone())
            .or_insert_with(Vec::new)
            .push(reward);
    }

    for rewards_vec in validator_rewards.values_mut() {
        rewards_vec.sort_by_key(|reward| reward.epoch);
    }

    let mut total_ending_lamports = 0u64;

    for validator in selected_validators {
        let ending_stake = if let Some(validator_reward_history) = validator_rewards.get(validator)
        {
            let final_stake = validator_reward_history.iter().fold(
                starting_stake_per_validator,
                |current_stake, epoch_reward| {
                    let new_stake = epoch_reward.stake_after_epoch(current_stake);
                    new_stake
                },
            );

            final_stake
        } else {
            starting_stake_per_validator
        };

        total_ending_lamports = total_ending_lamports
            .checked_add(ending_stake)
            .ok_or(CliError::ArithmeticError)?;
    }

    info!(
        "Total starting stake: {:.3} SOL, Total ending stake: {:.3} SOL",
        total_starting_lamports as f64 / LAMPORTS_PER_SOL as f64,
        total_ending_lamports as f64 / LAMPORTS_PER_SOL as f64
    );

    let cycle_result = RebalancingCycle {
        start_epoch: cycle_start_epoch,
        end_epoch: cycle_end_epoch,
        selected_validators: selected_validators.to_vec(),
        starting_total_lamports: total_starting_lamports,
        ending_total_lamports: total_ending_lamports,
        starting_stake_per_validator,
    };

    Ok((total_ending_lamports, cycle_result))
}
