use anyhow::Ok;

use crate::commands::BacktestArgs;

pub fn validator_score(parameters: &BacktestArgs, current_epoch: u16) -> u64 {
    1u64
}
