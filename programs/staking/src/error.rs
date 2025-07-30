use anchor_lang::prelude::*;

#[error_code]
pub enum StakingError {
    #[msg("Not enough time has passed to claim rewards.")]
    TooEarly,
}
