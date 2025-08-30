use crate::error::CliError;
use futures::future::try_join_all;
use jito_steward::{
    Config,
    constants::TVC_ACTIVATION_EPOCH,
    score::{instant_unstake_validator, validator_score},
};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use sqlx::{Pool, Postgres};
use stakenet_simulator_db::{
    cluster_history::ClusterHistory, cluster_history_entry::ClusterHistoryEntry,
    epoch_rewards::EpochRewards, validator_history::ValidatorHistory,
    validator_history_entry::ValidatorHistoryEntry,
};
use std::collections::HashMap;
use std::sync::Arc;
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
    let jito_cluster_history = Arc::new(jito_cluster_history);
    info!("Fetching all validator history entries...");
    let all_entries =
        ValidatorHistoryEntry::fetch_all_validator_history_entries(db_connection).await?;
    let mut entries_by_validator: HashMap<String, Vec<ValidatorHistoryEntry>> = HashMap::new();
    for entry in all_entries {
        entries_by_validator
            .entry(entry.vote_pubkey.clone())
            .or_insert_with(Vec::new)
            .push(entry);
    }
    let entries_by_validator = Arc::new(entries_by_validator);
    info!(
        "Grouped {} validators' history entries",
        entries_by_validator.len()
    );

    let mut top_validators = Vec::new();

    for current_epoch in simulation_start_epoch..simulation_end_epoch {
        let starting_stake_per_validator =
            lamports_after_staking / number_of_validator_delegations as u64;
        let current_cycle_end = std::cmp::min(
            current_cycle_start + steward_cycle_rate,
            simulation_end_epoch,
        );
        if (current_epoch - simulation_start_epoch) % steward_cycle_rate == 0 {
            top_validators = top_validators_for_epoch(
                &histories,
                &entries_by_validator,
                &jito_cluster_history,
                steward_config,
                current_epoch,
                number_of_validator_delegations,
            )
            .await?;

            // THE POSITION OF THIS DEPENDS ON THE LOGIC OF HOW THE STAKE IS REDISTRIBUTED IN AN
            // UNSTAKE.
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

        // TBD: WOULD WE RECALCULATE THIS IF THE TOP VALIDATORS ARE CALCULATED IN THE current_epoch ONLY?
        let validators_to_unstake = calculate_unstake_per_epoch(
            &top_validators,
            &histories,
            &entries_by_validator,
            &jito_cluster_history,
            steward_config,
            current_epoch,
        )
        .await?;

        info!(
            "{} validators unstaked for epoch {}",
            validators_to_unstake.len(),
            current_epoch
        );
    }

    Ok(rebalancing_cycles)
}

async fn calculate_unstake_per_epoch(
    selected_validators: &[String],
    histories: &[ValidatorHistory],
    entries_by_validator: &Arc<HashMap<String, Vec<ValidatorHistoryEntry>>>,
    jito_cluster_history: &Arc<JitoClusterHistory>,
    steward_config: &Config,
    epoch: u16,
) -> Result<Vec<String>, CliError> {
    info!(
        "Checking instant unstake for epoch {} on {} validators",
        epoch,
        selected_validators.len()
    );
    let epoch_start_slot = epoch as u64 * 432000;
    let unstake_tasks: Vec<_> = selected_validators
        .iter()
        .filter_map(|validator_vote_account| {
            histories
                .iter()
                .find(|vh| vh.vote_account == *validator_vote_account)
                .map(|validator_history| {
                    let validator_history = validator_history.clone();
                    let entries_by_validator = Arc::clone(entries_by_validator);
                    let jito_cluster_history = Arc::clone(jito_cluster_history);
                    let steward_config = steward_config.clone();
                    let vote_account = validator_vote_account.clone();
                    tokio::task::spawn_blocking(move || {
                        let unstake_result = calculate_instant_unstake(
                            validator_history,
                            &entries_by_validator,
                            &jito_cluster_history,
                            &steward_config,
                            epoch_start_slot,
                            epoch,
                        );
                        (vote_account, unstake_result)
                    })
                })
        })
        .collect();
    let unstake_results = try_join_all(unstake_tasks)
        .await
        .map_err(|e| CliError::TaskJoinError(e))?;

    let mut validators_to_unstake = Vec::new();

    for (vote_account, result) in unstake_results {
        match result {
            Ok(should_unstake) => {
                if should_unstake {
                    // UNCOMMENT AFTER WE HAVE THE DATA
                    // warn!(
                    //     "INSTANT UNSTAKE: Validator {} at epoch {}",
                    //     vote_account, epoch
                    // );
                    validators_to_unstake.push(vote_account);
                }
            }
            Err(e) => {
                error!(
                    "Error checking instant unstake for validator {} at epoch {}: {:?}",
                    vote_account, epoch, e
                );
            }
        }
    }
    Ok(validators_to_unstake)
}

async fn top_validators_for_epoch(
    histories: &[ValidatorHistory],
    entries_by_validator: &Arc<HashMap<String, Vec<ValidatorHistoryEntry>>>,
    jito_cluster_history: &Arc<JitoClusterHistory>,
    steward_config: &Config,
    scoring_epoch: u16,
    number_of_validators: usize,
) -> Result<Vec<String>, CliError> {
    info!("Scoring validators for epoch {}", scoring_epoch);

    let scoring_tasks: Vec<_> = histories
        .iter()
        .map(|validator_history| {
            let validator_history = validator_history.clone();
            let entries_by_validator = Arc::clone(entries_by_validator);
            let jito_cluster_history = Arc::clone(jito_cluster_history);
            let steward_config = steward_config.clone();
            tokio::task::spawn_blocking(move || {
                score_validator(
                    validator_history,
                    &entries_by_validator,
                    &jito_cluster_history,
                    &steward_config,
                    scoring_epoch,
                )
            })
        })
        .collect();

    let scoring_results = try_join_all(scoring_tasks)
        .await
        .map_err(|e| CliError::TaskJoinError(e))?;

    let mut scored_validators: Vec<(String, f64)> = scoring_results
        .into_iter()
        .filter_map(|result| result.ok())
        .collect();

    scored_validators.sort_by(|a, b| b.1.total_cmp(&a.1));

    // Get only the validators that have score more than zero, maximum number of validators can be "number_of_validators"
    let top_validators: Vec<String> = scored_validators
        .into_iter()
        .filter(|(_, score)| *score > 0.0)
        .take(number_of_validators)
        .map(|(vote_account, _score)| vote_account)
        .collect();

    Ok(top_validators)
}

pub fn score_validator(
    validator_history: ValidatorHistory,
    entries_by_validator: &HashMap<String, Vec<ValidatorHistoryEntry>>,
    jito_cluster_history: &JitoClusterHistory,
    steward_config: &Config,
    current_epoch: u16,
) -> Result<(String, f64), CliError> {
    let vote_account = validator_history.vote_account.clone();

    // Get entries for this validator from the pre-fetched map
    let mut entries = entries_by_validator
        .get(&vote_account)
        .cloned()
        .unwrap_or_default();

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
            // TODO: Uncomment when we get actual data (For now this is overwhelming)
            // error!("No rewards for validator: {}", validator);
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

fn calculate_instant_unstake(
    validator_history: ValidatorHistory,
    entries_by_validator: &HashMap<String, Vec<ValidatorHistoryEntry>>,
    jito_cluster_history: &JitoClusterHistory,
    config: &Config,
    epoch_start_slot: u64,
    current_epoch: u16,
) -> Result<bool, CliError> {
    let vote_account = validator_history.vote_account.clone();
    let mut entries = entries_by_validator
        .get(&vote_account)
        .cloned()
        .unwrap_or_default();

    let jito_validator_history = validator_history.convert_to_jito_validator_history(&mut entries);

    let unstake_result = instant_unstake_validator(
        &jito_validator_history,
        jito_cluster_history,
        config,
        epoch_start_slot,
        current_epoch,
        TVC_ACTIVATION_EPOCH,
    );

    match unstake_result {
        Ok(unstake) => Ok(unstake.instant_unstake),
        Err(_) => {
            error!(
                "Error calculating instant unstake for validator {}",
                jito_validator_history.vote_account
            );
            // TBD: Is there is an error in calculating, we should NOT unstake the validator?
            Ok(false)
        }
    }
}
