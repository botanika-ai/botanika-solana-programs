use anchor_lang::prelude::*;
use crate::shared_types::*;

/// Validation utilities for cross-program use
pub mod validation {
    use super::*;

    /// Validate staking amount
    pub fn validate_staking_amount(amount: u64) -> Result<()> {
        require!(amount > 0, BotanikaError::InvalidAmount);
        Ok(())
    }

    /// Validate staking level
    pub fn validate_staking_level(level: StakingLevel) -> Result<()> {
        match level {
            StakingLevel::Bronze | StakingLevel::Silver | 
            StakingLevel::Gold | StakingLevel::Platinum => Ok(()),
        }
    }

    /// Validate multiplier value
    pub fn validate_multiplier(multiplier: u64) -> Result<()> {
        require!(multiplier > 0, BotanikaError::InvalidMultiplier);
        require!(multiplier <= 10000, BotanikaError::InvalidMultiplier); // Max 10x
        Ok(())
    }

    /// Validate proof type
    pub fn validate_proof_type(proof_type: ProofType) -> Result<()> {
        match proof_type {
            ProofType::ProofOfStake | ProofType::ProofOfExecution |
            ProofType::ProofOfService | ProofType::ProofOfEfficiency => Ok(()),
        }
    }

    /// Check if lockup period is met
    pub fn check_lockup_period(staked_at: i64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let lockup_end = staked_at + LOCKUP_PERIOD;
        require!(now >= lockup_end, BotanikaError::LockupPeriodNotMet);
        Ok(())
    }

    /// Check if claim cooldown is met
    pub fn check_claim_cooldown(last_claimed_at: i64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let cooldown_end = last_claimed_at + CLAIM_COOLDOWN;
        require!(now >= cooldown_end, BotanikaError::ClaimCooldownNotMet);
        Ok(())
    }
} 