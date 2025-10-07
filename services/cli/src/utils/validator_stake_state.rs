use crate::error::CliError;

#[derive(Clone, Debug, Default)]
pub struct ValidatorStakeState {
    pub active: u64,
    pub activating: u64,
    pub deactivating: u64,
    pub target: u64,
}

impl ValidatorStakeState {
    pub fn total(&self) -> u64 {
        self.active + self.activating + self.deactivating
    }

    pub fn add_activating_stake(&mut self, amount: u64) {
        self.activating += amount;
    }

    pub fn add_deactivating_stake(&mut self, amount: u64) -> Result<(), CliError> {
        if self.active < amount {
            return Err(CliError::ArithmeticError);
        }
        self.active -= amount;
        self.deactivating += amount;
        Ok(())
    }

    /// Process epoch transition: activating->active, deactivating->removed
    pub fn process_epoch_transition(&mut self) {
        // Activating stake becomes active
        self.active += self.activating;
        self.activating = 0;
        // Deactivating stake is removed
        self.deactivating = 0;
    }

    /// This increases the activated stake by the `ratio` provided
    pub fn increase_activating_stake(&mut self, ratio: f64) -> Result<(), CliError> {
        if self.active == 0 {
            return Ok(());
        }
        let active_f64 = self.active as f64;
        let adjustment = active_f64 * ratio;
        let new_active = (active_f64 + adjustment).max(0.0) as u64;
        self.active = new_active;
        Ok(())
    }

    /// Apply rewards only to active stake
    pub fn apply_rewards(&mut self, reward_lamports: u64) {
        self.active += reward_lamports;
    }
}
