use anchor_lang::prelude::*;

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

#[account]
#[derive(Default)]
pub struct GovernanceState {
    /// Authority that can execute proposals
    pub authority: Pubkey,
    
    /// Current staking level multipliers
    pub multipliers: [u64; 4], // Bronze, Silver, Gold, Platinum
    
    /// Fee structure (in basis points)
    pub staking_fee_bps: u64,
    pub unstaking_fee_bps: u64,
    pub claim_fee_bps: u64,
    
    /// Proposal settings
    pub proposal_duration: i64,
    pub quorum_threshold: u64,
    
    /// Last updated timestamp
    pub last_updated: i64,
    
    /// Reserved space for future upgrades
    pub bump: u8,
    
    pub _reserved: [u8; 32],
}

impl GovernanceState {
    pub const SIZE: usize = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 32;
}

#[account]
#[derive(Default)]
pub struct Proposal {
    /// Proposal ID
    pub id: u64,
    
    /// Proposal type
    pub proposal_type: ProposalType,
    
    /// Current status
    pub status: ProposalStatus,
    
    /// Proposal data (serialized)
    pub data: Vec<u8>,
    
    /// Created timestamp
    pub created_at: i64,
    
    /// Voting end timestamp
    pub voting_ends_at: i64,
    
    /// Execution timestamp (if executed)
    pub executed_at: Option<i64>,
    
    /// Votes for
    pub votes_for: u64,
    
    /// Votes against
    pub votes_against: u64,
    
    /// Reserved space for future upgrades
    pub bump: u8,
    
    pub _reserved: [u8; 16],
}

impl Proposal {
    pub const SIZE: usize = 8 + 8 + 1 + 1 + 4 + 8 + 8 + 8 + 8 + 8 + 1 + 16;
}

impl Proposal {
    /// Check if proposal is active
    pub fn is_active(&self) -> bool {
        self.status == ProposalStatus::Active
    }
    
    /// Check if voting period has ended
    pub fn voting_ended(&self) -> bool {
        let now = Clock::get().unwrap().unix_timestamp;
        now > self.voting_ends_at
    }
    
    /// Check if proposal passed
    pub fn passed(&self) -> bool {
        self.votes_for > self.votes_against
    }
    
    /// Mark as executed
    pub fn mark_executed(&mut self) {
        self.status = ProposalStatus::Executed;
        self.executed_at = Some(Clock::get().unwrap().unix_timestamp);
    }
    
    /// Mark as expired
    pub fn mark_expired(&mut self) {
        self.status = ProposalStatus::Expired;
    }
} 