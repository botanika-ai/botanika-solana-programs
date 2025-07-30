use anchor_lang::prelude::*;
use crate::shared_types::*;

#[account]
#[derive(Default)]
pub struct StakingState {
    /// Program-level config: authority for staking config update
    pub admin: Pubkey,

    /// Mint of the staking token (e.g., BONSAI)
    pub staking_mint: Pubkey,

    /// Vault holding all staked tokens
    pub vault: Pubkey,

    /// Reward authority (can be a multisig or DAO program)
    pub reward_authority: Pubkey,

    /// Reward rate per slot (scaled, e.g., 1e6 == 1 token)
    pub reward_rate_per_slot: u64,

    /// Last updated slot (used to calculate reward delta)
    pub last_update_slot: u64,

    /// Total amount staked
    pub total_staked: u64,

    /// Current staking level multipliers (from governance)
    pub multipliers: [u64; 4], // Bronze, Silver, Gold, Platinum

    /// Lockup period in seconds
    pub lockup_period: i64,

    /// Claim cooldown in seconds
    pub claim_cooldown: i64,

    /// Reserved space for future upgrades
    pub bump: u8,

    pub _reserved: [u8; 32],
}

impl StakingState {
    pub const SIZE: usize = 8 + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 32 + 8 + 8 + 1 + 32;
}

#[account]
#[derive(Default)]
pub struct UserStake {
    /// The user who owns this stake account
    pub owner: Pubkey,

    /// The amount staked by this user
    pub amount: u64,

    /// Accumulated reward (claimable)
    pub reward_debt: u64,

    /// Last slot when reward was updated
    pub last_updated_slot: u64,

    /// Current staking level
    pub level: StakingLevel,

    /// Current staking status
    pub status: StakingStatus,

    /// Timestamp when staking started
    pub staked_at: i64,

    /// Last time rewards were claimed
    pub last_claimed_at: i64,

    /// Lockup end timestamp
    pub lockup_end: i64,

    /// Reserved space for future upgrades
    pub bump: u8,

    pub _reserved: [u8; 16],
}

impl UserStake {
    pub const SIZE: usize = 8 + 32 + 8 + 8 + 8 + 1 + 1 + 8 + 8 + 8 + 1 + 16;
}

impl UserStake {
    /// Calculate staking level based on amount
    pub fn calculate_level(&self) -> StakingLevel {
        match self.amount {
            0..=1000 => StakingLevel::Bronze,
            1001..=5000 => StakingLevel::Silver,
            5001..=10000 => StakingLevel::Gold,
            _ => StakingLevel::Platinum,
        }
    }

    /// Check if lockup period is met
    pub fn is_lockup_met(&self) -> bool {
        let now = Clock::get().unwrap().unix_timestamp;
        now >= self.lockup_end
    }

    /// Check if claim cooldown is met
    pub fn can_claim(&self) -> bool {
        let now = Clock::get().unwrap().unix_timestamp;
        now >= self.last_claimed_at + CLAIM_COOLDOWN
    }

    /// Get multiplier for current level
    pub fn get_multiplier(&self, state: &StakingState) -> u64 {
        match self.level {
            StakingLevel::Bronze => state.multipliers[0],
            StakingLevel::Silver => state.multipliers[1],
            StakingLevel::Gold => state.multipliers[2],
            StakingLevel::Platinum => state.multipliers[3],
        }
    }
}
