use anchor_lang::prelude::*;

#[error_code]
pub enum CliError {
    #[msg("ArithmeticError")]
    ArithmeticError,

    #[msg("StakeHistoryNotRecentEnough")]
    StakeHistoryNotRecentEnough,

    #[msg("ClusterHistoryNotRecentEnough")]
    ClusterHistoryNotRecentEnough,

    #[msg("VoteHistoryNotRecentEnough")]
    VoteHistoryNotRecentEnough,
}
