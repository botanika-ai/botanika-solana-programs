use anchor_lang::prelude::*;
use crate::shared_types::*;

/// Safe math utilities for cross-program use
pub mod math {
    use super::*;

    /// Safe addition with overflow check
    pub fn safe_add(a: u64, b: u64) -> Result<u64> {
        a.checked_add(b).ok_or(BotanikaError::Overflow)
    }

    /// Safe subtraction with underflow check
    pub fn safe_sub(a: u64, b: u64) -> Result<u64> {
        a.checked_sub(b).ok_or(BotanikaError::Overflow)
    }

    /// Safe multiplication with overflow check
    pub fn safe_mul(a: u64, b: u64) -> Result<u64> {
        a.checked_mul(b).ok_or(BotanikaError::Overflow)
    }

    /// Safe division with zero check
    pub fn safe_div(a: u64, b: u64) -> Result<u64> {
        require!(b > 0, BotanikaError::InvalidAmount);
        a.checked_div(b).ok_or(BotanikaError::Overflow)
    }

    /// Calculate reward based on amount, time, and multiplier
    pub fn calculate_reward(
        amount: u64,
        time_elapsed: i64,
        multiplier: u64,
    ) -> Result<u64> {
        let base_reward = safe_mul(amount, time_elapsed as u64)?;
        let multiplied_reward = safe_mul(base_reward, multiplier)?;
        let normalized_reward = safe_div(multiplied_reward, 1000)?;
        let annualized_reward = safe_div(normalized_reward, 365 * 24 * 60 * 60)?;
        Ok(annualized_reward)
    }

    /// Calculate fee amount
    pub fn calculate_fee(amount: u64, fee_bps: u64) -> Result<u64> {
        let fee = safe_mul(amount, fee_bps)?;
        safe_div(fee, 10000)
    }

    /// Calculate staking level from amount
    pub fn calculate_staking_level(amount: u64) -> StakingLevel {
        match amount {
            0..=1000 => StakingLevel::Bronze,
            1001..=5000 => StakingLevel::Silver,
            5001..=10000 => StakingLevel::Gold,
            _ => StakingLevel::Platinum,
        }
    }
} 