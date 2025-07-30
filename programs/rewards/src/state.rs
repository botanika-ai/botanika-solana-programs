use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ProofType {
    ProofOfStake,      // Staking-based rewards
    ProofOfExecution,   // Computational work
    ProofOfService,     // Node operation
    ProofOfEfficiency,  // Energy efficiency
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum RewardStatus {
    Pending,
    Available,
    Claimed,
    Expired,
}

#[account]
#[derive(Default)]
pub struct RewardPool {
    /// Authority that can submit rewards
    pub authority: Pubkey,
    
    /// Total rewards distributed
    pub total_rewards_distributed: u64,
    
    /// Reward token mint
    pub reward_mint: Pubkey,
    
    /// Reward vault token account
    pub reward_vault: Pubkey,
    
    /// Expiration period for rewards (in seconds)
    pub expiration_period: i64,
    
    /// Reserved space for future upgrades
    pub bump: u8,
    
    pub _reserved: [u8; 32],
}

impl RewardPool {
    pub const SIZE: usize = 8 + 32 + 8 + 32 + 32 + 8 + 1 + 32;
}

#[account]
#[derive(Default)]
pub struct RewardRecipient {
    /// User who can claim rewards
    pub user: Pubkey,
    
    /// Type of proof that earned the reward
    pub proof_type: ProofType,
    
    /// Amount of rewards earned
    pub amount: u64,
    
    /// Current status of the reward
    pub status: RewardStatus,
    
    /// Timestamp when reward was submitted
    pub submitted_at: i64,
    
    /// Timestamp when reward was claimed (if claimed)
    pub claimed_at: Option<i64>,
    
    /// Expiration timestamp
    pub expires_at: i64,
    
    /// Reserved space for future upgrades
    pub bump: u8,
    
    pub _reserved: [u8; 16],
}

impl RewardRecipient {
    pub const SIZE: usize = 8 + 32 + 1 + 8 + 1 + 8 + 8 + 8 + 1 + 16;
}

impl RewardRecipient {
    /// Check if reward is expired
    pub fn is_expired(&self) -> bool {
        let now = Clock::get().unwrap().unix_timestamp;
        now > self.expires_at
    }
    
    /// Check if reward can be claimed
    pub fn can_claim(&self) -> bool {
        self.status == RewardStatus::Available && !self.is_expired()
    }
    
    /// Mark reward as claimed
    pub fn mark_claimed(&mut self) {
        self.status = RewardStatus::Claimed;
        self.claimed_at = Some(Clock::get().unwrap().unix_timestamp);
    }
    
    /// Mark reward as expired
    pub fn mark_expired(&mut self) {
        self.status = RewardStatus::Expired;
    }
} 