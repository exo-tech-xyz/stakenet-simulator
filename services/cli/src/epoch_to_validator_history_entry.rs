use std::{cmp::Ordering, collections::HashMap, str::FromStr};

use anchor_lang::{prelude::Pubkey, solana_program::example_mocks::solana_sdk::system_program};
use stakenet_simulator_db::validator_history_entry::ValidatorHistoryEntry;
use validator_history::{self, CircBuf};

pub struct EpochToValdiatorHistoryEntry(pub HashMap<u16, ValidatorHistoryEntry>);

impl EpochToValdiatorHistoryEntry {
    pub fn get_latest(&self, current_epoch: u16) -> Option<&ValidatorHistoryEntry> {
        self.0.get(&current_epoch)
    }
    pub fn to_validator_history(self) -> validator_history::ValidatorHistory {
        let vote_account = self
            .0
            .values()
            .find(|x| x.vote_pubkey != system_program::ID.to_string())
            .unwrap()
            .vote_pubkey
            .clone();
        let mut validator_history: validator_history::ValidatorHistory =
            validator_history::ValidatorHistory {
                struct_version: 0,
                vote_account: Pubkey::from_str(&vote_account).unwrap(),
                // TODO: Need to pull this from DB
                index: 0,
                // dummy data
                bump: 0,
                _padding0: [0u8; 7],
                // REVIEW: dummy data
                last_ip_timestamp: 0,
                // REVIEW: dummy data
                last_version_timestamp: 0,
                _padding1: [0u8; 232],
                history: CircBuf::default(),
            };
        let mut entries: Vec<ValidatorHistoryEntry> = self.0.into_values().collect();
        // Sort entries by epoch, low to high
        entries.sort_by(|a, b| {
            a.validator_history_entry
                .epoch
                .cmp(&b.validator_history_entry.epoch)
        });
        // Loop through sorted entries insert into ValidatorHistory
        for entry in entries.into_iter() {
            if let Some(last_entry) = validator_history.history.last_mut() {
                match last_entry.epoch.cmp(&entry.validator_history_entry.epoch) {
                    Ordering::Equal => {
                        *last_entry = entry.validator_history_entry;
                    }
                    Ordering::Greater => {
                        *last_entry = entry.validator_history_entry;
                    }
                    Ordering::Less => {
                        validator_history
                            .history
                            .push(entry.validator_history_entry);
                    }
                }
            }
        }

        validator_history
    }
}
