use anchor_lang::prelude::*;

#[error_code]
pub enum RewardsError {
    #[msg("Invalid proof type")]
    InvalidProofType,
    
    #[msg("Reward already claimed")]
    RewardAlreadyClaimed,
    
    #[msg("Reward expired")]
    RewardExpired,
    
    #[msg("Reward not available")]
    RewardNotAvailable,
    
    #[msg("Invalid authority")]
    InvalidAuthority,
    
    #[msg("Overflow in calculation")]
    Overflow,
    
    #[msg("Invalid amount")]
    InvalidAmount,
    
    #[msg("Reward pool not initialized")]
    RewardPoolNotInitialized,
} 