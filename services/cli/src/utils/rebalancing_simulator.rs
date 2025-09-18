use crate::{error::CliError, utils::ValidatorStakeState};
use futures::future::try_join_all;
use jito_steward::{
    Config,
    constants::TVC_ACTIVATION_EPOCH,
    score::{instant_unstake_validator, validator_score},
};
use num_traits::ToPrimitive;
use rand::prelude::IndexedRandom;
use rand::rng;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use sqlx::{Pool, Postgres};
use stakenet_simulator_db::{
    active_stake_jito_sol::ActiveStakeJitoSol, cluster_history::ClusterHistory,
    cluster_history_entry::ClusterHistoryEntry, epoch_rewards::EpochRewards,
    validator_history::ValidatorHistory, validator_history_entry::ValidatorHistoryEntry,
    withdraw_and_deposits::WithdrawsAndDeposits,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{error, info};
use validator_history::ClusterHistory as JitoClusterHistory;

#[derive(Clone, Debug)]
pub struct RebalancingCycle {
    pub starting_total_lamports: u64,
    pub ending_total_lamports: u64,
}

#[derive(Debug, Clone)]
pub struct EpochWithdrawDepositStakeData {
    pub withdraw_stake: f64,
    pub deposit_stake: f64,
    pub active_balance: f64,
}

#[derive(Clone, Debug)]
pub struct ValidatorWithScore {
    pub vote_account: String,
    pub score: f64,
}

pub struct RebalancingSimulator {
    pub steward_config: Config,
    pub simulation_start_epoch: u16,
    pub simulation_end_epoch: u16,
    pub steward_cycle_rate: u16,
    pub number_of_validator_delegations: usize,
    pub instant_unstake_cap_bps: u32,

    pub validator_stake_states: HashMap<String, ValidatorStakeState>,
    pub validator_scores: HashMap<String, f64>,
    pub current_cycle_start: u16,
    pub total_lamports_staked: u64,
    pub rebalancing_cycles: Vec<RebalancingCycle>,
    pub top_validators: Vec<ValidatorWithScore>,

    pub histories: Vec<ValidatorHistory>,
    pub jito_cluster_history: Arc<JitoClusterHistory>,
    pub entries_by_validator: Arc<HashMap<String, Vec<ValidatorHistoryEntry>>>,
    pub epoch_map: HashMap<u64, Vec<EpochWithdrawDepositStakeData>>,
}

impl RebalancingSimulator {
    /// This function is responsible for querying all the data required for the simulation cycle.
    pub async fn new(
        db_connection: &Pool<Postgres>,
        steward_config: Config,
        simulation_start_epoch: u16,
        simulation_end_epoch: u16,
        steward_cycle_rate: u16,
        number_of_validator_delegations: usize,
        instant_unstake_cap_bps: u32,
        validator_historical_start_offset: u16,
    ) -> Result<Self, CliError> {
        info!("Initializing rebalancing simulator...");

        let histories = ValidatorHistory::fetch_all(db_connection).await?;
        let cluster_history = ClusterHistory::fetch(db_connection).await?;
        let cluster_history_entries = ClusterHistoryEntry::fetch_all(db_connection).await?;
        let jito_cluster_history =
            Arc::new(cluster_history.convert_to_jito_cluster_history(cluster_history_entries));

        info!("Fetching all validator history entries...");
        let all_entries = ValidatorHistoryEntry::fetch_all_records_between_epochs(
            db_connection,
            simulation_start_epoch
                .saturating_sub(validator_historical_start_offset)
                .into(),
            simulation_end_epoch.into(),
        )
        .await?;

        let withdraws_and_deposits_stakes = WithdrawsAndDeposits::get_details_for_epoch_range(
            db_connection,
            simulation_start_epoch.into(),
            simulation_end_epoch.into(),
        )
        .await?;

        let active_stake = ActiveStakeJitoSol::get_active_stakes_for_epoch_range(
            db_connection,
            simulation_start_epoch.into(),
            simulation_end_epoch.into(),
        )
        .await?;

        let epoch_map = Self::build_epoch_map(withdraws_and_deposits_stakes, active_stake);
        let entries_by_validator = Self::build_entries_by_validator(all_entries);

        info!(
            "Grouped {} validators' history entries",
            entries_by_validator.len()
        );

        // start with one sol per validator
        let total_lamports_staked = LAMPORTS_PER_SOL
            .checked_mul(number_of_validator_delegations as u64)
            .ok_or(CliError::ArithmeticError)?;

        Ok(Self {
            steward_config,
            simulation_start_epoch,
            simulation_end_epoch,
            steward_cycle_rate,
            number_of_validator_delegations,
            instant_unstake_cap_bps,
            validator_stake_states: HashMap::new(),
            validator_scores: HashMap::new(),
            current_cycle_start: simulation_start_epoch,
            total_lamports_staked,
            rebalancing_cycles: Vec::new(),
            top_validators: Vec::new(),
            histories,
            jito_cluster_history,
            entries_by_validator: Arc::new(entries_by_validator),
            epoch_map,
        })
    }

    /// Main simulation entry point
    pub async fn run_simulation(
        &mut self,
        db_connection: &Pool<Postgres>,
    ) -> Result<Vec<RebalancingCycle>, CliError> {
        let mut cycle_starting_lamports = 0u64;

        for current_epoch in self.simulation_start_epoch..self.simulation_end_epoch {
            info!("Processing epoch {}", current_epoch);

            // for all validators, put all the activating sol in the previous epoch as active and remove all the
            // deactivating sol
            self.process_epoch_transitions();

            let is_rebalancing_epoch = self.is_rebalancing_epoch(current_epoch);
            // filter the validator entries to get only the entries that are before the current epoch
            let current_epoch_entries = self.get_current_epoch_entries(current_epoch);

            if is_rebalancing_epoch {
                // end the previous steward cycle and starts a new one every `rebalancing epoch`
                cycle_starting_lamports = self
                    .process_steward_cycle(
                        &current_epoch_entries,
                        current_epoch,
                        cycle_starting_lamports,
                    )
                    .await?;
            }

            if !self.top_validators.is_empty() {
                // process normal epoch cycle
                self.process_epoch_cycle(
                    db_connection,
                    &current_epoch_entries,
                    current_epoch,
                    is_rebalancing_epoch,
                )
                .await?;
            }
        }

        self.finalize_simulation(cycle_starting_lamports);
        Ok(self.rebalancing_cycles.clone())
    }

    fn process_epoch_transitions(&mut self) {
        let mut validators_to_completely_remove = Vec::new();

        for (vote_account, stake_state) in self.validator_stake_states.iter_mut() {
            stake_state.process_epoch_transition();

            if stake_state.total() == 0 {
                validators_to_completely_remove.push(vote_account.clone());
            }
        }

        for vote_account in validators_to_completely_remove {
            self.validator_stake_states.remove(&vote_account);
        }
    }

    /// checks if the current epoch is the start fo a new steward cycle
    fn is_rebalancing_epoch(&self, current_epoch: u16) -> bool {
        (current_epoch - self.simulation_start_epoch) % self.steward_cycle_rate == 0
    }

    /// From all the validator entries, filter only the entires that are before the current epoch
    fn get_current_epoch_entries(
        &self,
        current_epoch: u16,
    ) -> Arc<HashMap<String, Vec<ValidatorHistoryEntry>>> {
        let current_epoch_entries: HashMap<String, Vec<ValidatorHistoryEntry>> = self
            .entries_by_validator
            .iter()
            .map(|(vote_pubkey, entries)| {
                let mut filtered_entries: Vec<ValidatorHistoryEntry> = entries
                    .iter()
                    .filter(|entry| entry.validator_history_entry.epoch <= current_epoch)
                    .cloned()
                    .collect();

                filtered_entries.sort_by(|a, b| {
                    b.validator_history_entry
                        .epoch
                        .cmp(&a.validator_history_entry.epoch)
                });

                (vote_pubkey.clone(), filtered_entries)
            })
            .collect();

        Arc::new(current_epoch_entries)
    }

    /// Starts a new steward cycle, called when a epoch is `rebalancing_epoch`
    async fn process_steward_cycle(
        &mut self,
        current_epoch_entries: &Arc<HashMap<String, Vec<ValidatorHistoryEntry>>>,
        current_epoch: u16,
        cycle_starting_lamports: u64,
    ) -> Result<u64, CliError> {
        let current_cycle_end = std::cmp::min(
            self.current_cycle_start + self.steward_cycle_rate,
            self.simulation_end_epoch,
        );

        if self.current_cycle_start > self.simulation_start_epoch {
            self.complete_cycle(cycle_starting_lamports);
            self.validator_stake_states.clear();
        }

        self.top_validators = self
            .select_top_validators(current_epoch_entries, current_epoch)
            .await?;

        let new_cycle_starting_lamports = self.rebalance_stakes();

        self.current_cycle_start = current_cycle_end;
        Ok(new_cycle_starting_lamports)
    }

    /// process normal epoch cycle
    async fn process_epoch_cycle(
        &mut self,
        db_connection: &Pool<Postgres>,
        current_epoch_entries: &Arc<HashMap<String, Vec<ValidatorHistoryEntry>>>,
        current_epoch: u16,
        is_rebalancing_epoch: bool,
    ) -> Result<(), CliError> {
        // Factor in deposit/withdraws of the stakes
        self.apply_epoch_stake_changes(current_epoch)?;

        // We won't calculate instant unstakes in the epoch that asteward cycle starts
        if !is_rebalancing_epoch {
            self.handle_epoch_instant_unstaking(current_epoch_entries, current_epoch)
                .await?;
        }

        self.simulate_epoch_returns(db_connection, current_epoch)
            .await?;

        Ok(())
    }

    /// stores the result of the last steward cycle in the struct and updates the total lamports staked
    fn complete_cycle(&mut self, cycle_starting_lamports: u64) {
        let cycle_ending_lamports = self
            .validator_stake_states
            .values()
            .map(|state| state.total())
            .sum::<u64>();

        let cycle_result = RebalancingCycle {
            starting_total_lamports: cycle_starting_lamports,
            ending_total_lamports: cycle_ending_lamports,
        };

        info!(
            "Completed cycle: {:.3} SOL -> {:.3} SOL (return: {:.2}%)",
            cycle_starting_lamports as f64 / LAMPORTS_PER_SOL as f64,
            cycle_ending_lamports as f64 / LAMPORTS_PER_SOL as f64,
            ((cycle_ending_lamports as f64 / cycle_starting_lamports as f64) - 1.0) * 100.0
        );

        self.rebalancing_cycles.push(cycle_result);
        self.total_lamports_staked = cycle_ending_lamports;
    }

    /// spawns new `tokio::task` for all the validators, calculates their score
    /// and finds the top `self.number_of_validator_delegations` validators
    async fn select_top_validators(
        &self,
        current_epoch_entries: &Arc<HashMap<String, Vec<ValidatorHistoryEntry>>>,
        current_epoch: u16,
    ) -> Result<Vec<ValidatorWithScore>, CliError> {
        info!("Scoring validators for epoch {}", current_epoch);

        let scoring_tasks: Vec<_> = self
            .histories
            .iter()
            .map(|validator_history| {
                let validator_history = validator_history.clone();
                let entries_by_validator = Arc::clone(current_epoch_entries);
                let jito_cluster_history = Arc::clone(&self.jito_cluster_history);
                let steward_config = self.steward_config.clone();

                tokio::task::spawn_blocking(move || {
                    Self::score_validator(
                        validator_history,
                        &entries_by_validator,
                        &jito_cluster_history,
                        &steward_config,
                        current_epoch,
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

        let top_validators: Vec<ValidatorWithScore> = scored_validators
            .into_iter()
            .filter(|(_, score)| *score > 0.0)
            .take(self.number_of_validator_delegations)
            .map(|(vote_account, score)| ValidatorWithScore {
                vote_account,
                score,
            })
            .collect();

        Ok(top_validators)
    }

    /// rebalance the stakes from the validators
    fn rebalance_stakes(&mut self) -> u64 {
        let current_total_stake = self
            .validator_stake_states
            .values()
            .map(|state| state.total())
            .sum::<u64>();

        let new_validator_set: HashSet<String> = self
            .top_validators
            .iter()
            .map(|v| v.vote_account.clone())
            .collect();

        self.remove_validators_not_in_set(&new_validator_set);

        let target_total = if current_total_stake > 0 {
            current_total_stake
        } else {
            self.total_lamports_staked
        };

        self.redistribute_stakes(target_total);

        target_total
    }

    fn remove_validators_not_in_set(&mut self, new_validator_set: &HashSet<String>) {
        let validators_to_remove: Vec<String> = self
            .validator_stake_states
            .keys()
            .filter(|vote_account| !new_validator_set.contains(*vote_account))
            .cloned()
            .collect();

        for vote_account in validators_to_remove {
            if let Some(mut stake_state) = self.validator_stake_states.remove(&vote_account) {
                let total_stake = stake_state.total();

                if total_stake == 0 {
                    // If no stake, don't re-insert at all
                    continue;
                }

                stake_state.deactivating = total_stake;
                stake_state.active = 0;
                stake_state.activating = 0;
                self.validator_stake_states
                    .insert(vote_account, stake_state);
            }
        }
    }

    /// This function checks if the current total of the activating stake is greater than the target
    /// if greater, unstakes the differerence, puts that in deactivating and vice versa
    fn redistribute_stakes(&mut self, target_total: u64) {
        let stake_per_validator: u64 = target_total / self.top_validators.len() as u64;

        self.validator_scores.clear();
        for validator in &self.top_validators {
            self.validator_scores
                .insert(validator.vote_account.clone(), validator.score);

            let current_state = self
                .validator_stake_states
                .entry(validator.vote_account.clone())
                .or_insert_with(ValidatorStakeState::default);

            let current_total = current_state.total();

            if current_total < stake_per_validator {
                let deficit = stake_per_validator - current_total;
                current_state.add_activating_stake(deficit);
                info!(
                    "Adding {:.3} SOL activating stake to validator {} (current: {:.3} SOL, target: {:.3} SOL)",
                    deficit as f64 / LAMPORTS_PER_SOL as f64,
                    validator.vote_account,
                    current_total as f64 / LAMPORTS_PER_SOL as f64,
                    stake_per_validator as f64 / LAMPORTS_PER_SOL as f64
                );
            } else if current_total > stake_per_validator {
                let excess = current_total - stake_per_validator;
                if let Err(e) = current_state.add_deactivating_stake(excess) {
                    error!("Failed to add deactivating stake: {:?}", e);
                }
                info!(
                    "Deactivating {:.3} SOL from validator {} (current: {:.3} SOL, target: {:.3} SOL)",
                    excess as f64 / LAMPORTS_PER_SOL as f64,
                    validator.vote_account,
                    current_total as f64 / LAMPORTS_PER_SOL as f64,
                    stake_per_validator as f64 / LAMPORTS_PER_SOL as f64
                );
            }
        }

        info!(
            "Rebalanced to {} validators with target {:.3} SOL each, total: {:.3} SOL",
            self.top_validators.len(),
            stake_per_validator as f64 / LAMPORTS_PER_SOL as f64,
            target_total as f64 / LAMPORTS_PER_SOL as f64
        );
    }

    /// This functions takes random validators to factor in manual withdraw and deposit of stakes
    fn apply_epoch_stake_changes(&mut self, current_epoch: u16) -> Result<(), CliError> {
        let current_epoch_u64 = current_epoch as u64;

        if let Some(epoch_data_vec) = self.epoch_map.get(&current_epoch_u64) {
            let num_records = epoch_data_vec.len();
            if num_records == 0 {
                return Ok(());
            }

            let validator_accounts: Vec<String> =
                self.validator_stake_states.keys().cloned().collect();
            if validator_accounts.is_empty() {
                return Ok(());
            }

            let mut rng = rng();
            let selected_validators: Vec<String> = (0..num_records)
                .map(|_| {
                    validator_accounts
                        .choose(&mut rng)
                        .unwrap_or(&validator_accounts[0])
                        .clone()
                })
                .collect();

            info!(
                "Epoch {}: Applying stake changes to {} randomly selected validators from {} total validators",
                current_epoch,
                num_records,
                validator_accounts.len()
            );

            for (validator_account, epoch_data) in
                selected_validators.iter().zip(epoch_data_vec.iter())
            {
                if let Some(stake_state) = self.validator_stake_states.get_mut(validator_account) {
                    if epoch_data.active_balance == 0.0 {
                        continue;
                    }

                    let net_stake_change = epoch_data.deposit_stake - epoch_data.withdraw_stake;
                    // calculate the ratio of the stake/unstake of that epoch to the total active balance of the epoch.
                    // since we are using 1 Sol as a initial balance for every validator, this would be the effective stake/unstake we can do to the validator.
                    let stake_change_ratio = net_stake_change / epoch_data.active_balance;
                    let old_active = stake_state.active;

                    stake_state.apply_stake_change(stake_change_ratio)?;
                    let new_active = stake_state.active;

                    info!(
                        "Epoch {}: Adjusted validator {} active stake by {:.6} SOL ({:.2}% change) - Active: {:.6} -> {:.6} SOL",
                        current_epoch,
                        validator_account,
                        (new_active as i64 - old_active as i64) as f64 / LAMPORTS_PER_SOL as f64,
                        stake_change_ratio * 100.0,
                        old_active as f64 / LAMPORTS_PER_SOL as f64,
                        new_active as f64 / LAMPORTS_PER_SOL as f64
                    );
                }
            }
        }

        // updating the total lamports staked
        self.total_lamports_staked = self
            .validator_stake_states
            .values()
            .map(|state| state.total())
            .sum::<u64>();

        Ok(())
    }

    /// Calculate the validators that need to be unstaked in an epoch and then unstakes them
    async fn handle_epoch_instant_unstaking(
        &mut self,
        current_epoch_entries: &Arc<HashMap<String, Vec<ValidatorHistoryEntry>>>,
        current_epoch: u16,
    ) -> Result<(), CliError> {
        let current_validator_list: Vec<String> = self
            .top_validators
            .iter()
            .map(|v| v.vote_account.clone())
            .collect();

        let validators_to_unstake = self
            .calculate_unstake_per_epoch(
                &current_validator_list,
                current_epoch_entries,
                current_epoch,
            )
            .await?;

        if !validators_to_unstake.is_empty() {
            self.handle_instant_unstaking(&validators_to_unstake)?;
        }

        Ok(())
    }

    /// Spins up a new `tokio::task` to calculate the unstake score for all the selected validators
    /// returns a array of pubkeys of all the unstaked validators
    async fn calculate_unstake_per_epoch(
        &self,
        selected_validators: &[String],
        entries_by_validator: &Arc<HashMap<String, Vec<ValidatorHistoryEntry>>>,
        epoch: u16,
    ) -> Result<Vec<String>, CliError> {
        let epoch_start_slot = epoch as u64 * 432_000;
        let unstake_tasks: Vec<_> = selected_validators
            .iter()
            .filter_map(|validator_vote_account| {
                self.histories
                    .iter()
                    .find(|vh| vh.vote_account == *validator_vote_account)
                    .map(|validator_history| {
                        let validator_history = validator_history.clone();
                        let entries_by_validator = Arc::clone(entries_by_validator);
                        let jito_cluster_history = Arc::clone(&self.jito_cluster_history);
                        let steward_config = self.steward_config.clone();
                        let vote_account = validator_vote_account.clone();

                        tokio::task::spawn_blocking(move || {
                            let unstake_result = Self::calculate_instant_unstake(
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

    /// Unstakes the validators that need to be unstaked
    /// Takes into account the `instant_unstake_cap_bps` as the maximum amount that can be unstaked
    /// Unstakes the lower scored validators first
    /// Stake is put in deactivating and isn't instantly removed. Will be removed in the next epoch
    fn handle_instant_unstaking(
        &mut self,
        validators_to_unstake: &[String],
    ) -> Result<(), CliError> {
        let max_unstake_amount =
            (self.total_lamports_staked as u128 * self.instant_unstake_cap_bps as u128 / 10000)
                .min(u64::MAX as u128) as u64;

        let mut validators_with_scores: Vec<(String, f64, u64)> = validators_to_unstake
            .iter()
            .filter_map(|vote_account| {
                let score = self
                    .top_validators
                    .iter()
                    .find(|v| &v.vote_account == vote_account)
                    .map(|v| v.score)
                    .unwrap_or(0.0);

                self.validator_stake_states
                    .get(vote_account)
                    .map(|state| (vote_account.clone(), score, state.total()))
            })
            .collect();

        // sorting all the validators to be unstaked by scores
        validators_with_scores
            .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut actual_validators_to_unstake = Vec::new();
        let mut total_unstaked_amount = 0u64;

        for (vote_account, _score, total_balance) in validators_with_scores {
            let potential_unstake = total_unstaked_amount + total_balance;
            if potential_unstake <= max_unstake_amount {
                actual_validators_to_unstake.push(vote_account);
                total_unstaked_amount += total_balance;
            } else {
                break;
            }
        }

        // Deactivate stake from unstaked validators
        for validator in &actual_validators_to_unstake {
            if let Some(stake_state) = self.validator_stake_states.get_mut(validator) {
                let total_stake = stake_state.total();
                stake_state.add_deactivating_stake(stake_state.active)?;
                stake_state.deactivating += stake_state.activating;
                stake_state.activating = 0;

                info!(
                    "Instant unstaking: moved {:.3} SOL to deactivating for validator {}",
                    total_stake as f64 / LAMPORTS_PER_SOL as f64,
                    validator
                );
            }
        }

        // Redistribute to remaining validators
        if total_unstaked_amount > 0 {
            self.redistribute_unstaked_amount(total_unstaked_amount, &actual_validators_to_unstake);
        }

        Ok(())
    }

    /// Use the unstaked amount in the rebalancing and put it to the remaining validators.
    /// Amount isn't actually active yet as it will be activated in the next epoch. It will be added in the activating stake for that epoch
    fn redistribute_unstaked_amount(
        &mut self,
        total_unstaked_amount: u64,
        unstaked_validators: &[String],
    ) {
        let remaining_validators: Vec<_> = self
            .top_validators
            .iter()
            .filter(|v| !unstaked_validators.contains(&v.vote_account))
            .collect();

        if !remaining_validators.is_empty() {
            let stake_per_remaining_validator =
                total_unstaked_amount / remaining_validators.len() as u64;

            for validator in &remaining_validators {
                if let Some(stake_state) =
                    self.validator_stake_states.get_mut(&validator.vote_account)
                {
                    stake_state.add_activating_stake(stake_per_remaining_validator);
                }
            }

            info!(
                "Instant unstaking: redistributing {:.3} SOL as activating stake to {} remaining validators ({:.3} SOL each)",
                total_unstaked_amount as f64 / LAMPORTS_PER_SOL as f64,
                remaining_validators.len(),
                stake_per_remaining_validator as f64 / LAMPORTS_PER_SOL as f64
            );
        }
    }

    /// This function calculates the total returns before and after a epoch, and update the total lamports staked
    /// based on the rewards of the validators
    async fn simulate_epoch_returns(
        &mut self,
        db_connection: &Pool<Postgres>,
        current_epoch: u16,
    ) -> Result<(), CliError> {
        let total_before_rewards = self
            .validator_stake_states
            .values()
            .map(|state| state.total())
            .sum::<u64>();

        let validator_list: Vec<String> = self.validator_stake_states.keys().cloned().collect();
        let rewards = EpochRewards::fetch_for_single_epoch(
            db_connection,
            &validator_list,
            current_epoch.into(),
        )
        .await?;

        for reward in rewards {
            if let Some(stake_state) = self.validator_stake_states.get_mut(&reward.vote_pubkey) {
                if stake_state.active > 0 {
                    let reward_amount =
                        reward.stake_after_epoch(stake_state.active) - stake_state.active;
                    stake_state.apply_rewards(reward_amount);
                }
            }
        }

        let total_after_rewards = self
            .validator_stake_states
            .values()
            .map(|state| state.total())
            .sum::<u64>();

        self.total_lamports_staked = total_after_rewards;

        let active_stake_total = self
            .validator_stake_states
            .values()
            .map(|state| state.active)
            .sum::<u64>();

        info!(
            "Epoch {} returns: {:.6} SOL -> {:.6} SOL (gain: {:.6} SOL) - Active stake: {:.6} SOL",
            current_epoch,
            total_before_rewards as f64 / LAMPORTS_PER_SOL as f64,
            total_after_rewards as f64 / LAMPORTS_PER_SOL as f64,
            (total_after_rewards - total_before_rewards) as f64 / LAMPORTS_PER_SOL as f64,
            active_stake_total as f64 / LAMPORTS_PER_SOL as f64
        );

        Ok(())
    }

    /// Pushes the final rebalancing cycle
    fn finalize_simulation(&mut self, cycle_starting_lamports: u64) {
        if !self.validator_stake_states.is_empty() {
            let final_cycle_ending_lamports = self
                .validator_stake_states
                .values()
                .map(|state| state.total())
                .sum::<u64>();

            let final_cycle_result = RebalancingCycle {
                starting_total_lamports: cycle_starting_lamports,
                ending_total_lamports: final_cycle_ending_lamports,
            };

            self.rebalancing_cycles.push(final_cycle_result);
        }
    }

    /// This returns a hashmap of validator votekey to it's entries in the db
    fn build_entries_by_validator(
        all_entries: Vec<ValidatorHistoryEntry>,
    ) -> HashMap<String, Vec<ValidatorHistoryEntry>> {
        let mut entries_by_validator: HashMap<String, Vec<ValidatorHistoryEntry>> = HashMap::new();
        for entry in all_entries {
            entries_by_validator
                .entry(entry.vote_pubkey.clone())
                .or_insert_with(Vec::new)
                .push(entry);
        }
        entries_by_validator
    }

    /// This reutrns the hashap of manual withdraws and deposits of stakes epochwise
    fn build_epoch_map(
        withdraws_and_deposits: Vec<WithdrawsAndDeposits>,
        active_stake: Vec<ActiveStakeJitoSol>,
    ) -> HashMap<u64, Vec<EpochWithdrawDepositStakeData>> {
        let mut epoch_map: HashMap<u64, Vec<EpochWithdrawDepositStakeData>> = HashMap::new();
        let mut active_by_epoch: HashMap<u64, f64> = HashMap::new();

        for stake in active_stake {
            let balance = stake.balance.to_f64().unwrap_or(0.0);
            *active_by_epoch.entry(stake.epoch).or_insert(0.0) += balance;
        }

        for wd in withdraws_and_deposits {
            let active_balance = active_by_epoch.get(&wd.epoch).cloned().unwrap_or(0.0);

            epoch_map.entry(wd.epoch).or_insert_with(Vec::new).push(
                EpochWithdrawDepositStakeData {
                    withdraw_stake: wd.withdraw_stake.to_f64().unwrap_or(0.0),
                    deposit_stake: wd.deposit_stake.to_f64().unwrap_or(0.0),
                    active_balance,
                },
            );
        }

        epoch_map
    }

    fn score_validator(
        validator_history: ValidatorHistory,
        entries_by_validator: &HashMap<String, Vec<ValidatorHistoryEntry>>,
        jito_cluster_history: &JitoClusterHistory,
        steward_config: &Config,
        current_epoch: u16,
    ) -> Result<(String, f64), CliError> {
        let vote_account = validator_history.vote_account.clone();

        let mut entries = entries_by_validator
            .get(&vote_account)
            .cloned()
            .unwrap_or_default();

        let jito_validator_history =
            validator_history.convert_to_jito_validator_history(&mut entries);

        let score_result = validator_score(
            &jito_validator_history,
            jito_cluster_history,
            steward_config,
            current_epoch,
            TVC_ACTIVATION_EPOCH,
        );

        match score_result {
            Ok(score) => Ok((vote_account, score.score)),
            Err(_) => Ok((vote_account, 0.0)),
        }
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

        let jito_validator_history =
            validator_history.convert_to_jito_validator_history(&mut entries);

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
                Ok(false)
            }
        }
    }
}
