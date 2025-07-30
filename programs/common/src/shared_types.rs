use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

// Shared types for cross-program communication

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum StakingLevel {
    Bronze,    // 0-1000 BONSAI
    Silver,    // 1001-5000 BONSAI  
    Gold,      // 5001-10000 BONSAI
    Platinum,  // 10001+ BONSAI
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum StakingStatus {
    Active,
    Locked,
    Unstaking,
}

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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ProposalStatus {
    Draft,
    Active,
    Approved,
    Rejected,
    Executed,
    Expired,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ProposalType {
    MultiplierChange,
    FeeStructure,
    RewardPool,
    EmergencyAction,
}

// Shared constants
pub const LOCKUP_PERIOD: i64 = 7 * 24 * 60 * 60; // 7 days
pub const CLAIM_COOLDOWN: i64 = 24 * 60 * 60;     // 24 hours
pub const PROPOSAL_DURATION: i64 = 3 * 24 * 60 * 60; // 3 days

pub const BASE_MULTIPLIER: u64 = 1000; // 1.0x
pub const BRONZE_MULTIPLIER: u64 = 1200; // 1.2x
pub const SILVER_MULTIPLIER: u64 = 1500; // 1.5x
pub const GOLD_MULTIPLIER: u64 = 2000;   // 2.0x
pub const PLATINUM_MULTIPLIER: u64 = 3000; // 3.0x

pub const STAKING_FEE_BPS: u64 = 50;    // 0.5%
pub const UNSTAKING_FEE_BPS: u64 = 100;  // 1.0%
pub const CLAIM_FEE_BPS: u64 = 25;       // 0.25%

// Shared account structures for cross-program calls
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct StakingInfo {
    pub user: Pubkey,
    pub amount: u64,
    pub level: StakingLevel,
    pub status: StakingStatus,
    pub staked_at: i64,
    pub last_claimed_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RewardInfo {
    pub user: Pubkey,
    pub proof_type: ProofType,
    pub amount: u64,
    pub status: RewardStatus,
    pub submitted_at: i64,
    pub claimed_at: Option<i64>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct GovernanceInfo {
    pub authority: Pubkey,
    pub multipliers: [u64; 4], // Bronze, Silver, Gold, Platinum
    pub last_updated: i64,
}

// Error types for cross-program communication
#[error_code]
pub enum BotanikaError {
    #[msg("Invalid staking level")]
    InvalidStakingLevel,
    #[msg("Insufficient stake amount")]
    InsufficientStake,
    #[msg("Lockup period not met")]
    LockupPeriodNotMet,
    #[msg("Claim cooldown not met")]
    ClaimCooldownNotMet,
    #[msg("Invalid proof type")]
    InvalidProofType,
    #[msg("Reward already claimed")]
    RewardAlreadyClaimed,
    #[msg("Proposal not active")]
    ProposalNotActive,
    #[msg("Insufficient voting power")]
    InsufficientVotingPower,
    #[msg("Overflow in calculation")]
    Overflow,
    #[msg("Invalid authority")]
    InvalidAuthority,
} 