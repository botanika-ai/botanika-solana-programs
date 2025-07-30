use anchor_lang::prelude::*;

#[error_code]
pub enum GovernanceError {
    #[msg("Invalid authority")]
    InvalidAuthority,
    
    #[msg("Proposal not active")]
    ProposalNotActive,
    
    #[msg("Proposal already executed")]
    ProposalAlreadyExecuted,
    
    #[msg("Proposal expired")]
    ProposalExpired,
    
    #[msg("Voting period not ended")]
    VotingPeriodNotEnded,
    
    #[msg("Insufficient voting power")]
    InsufficientVotingPower,
    
    #[msg("Invalid proposal data")]
    InvalidProposalData,
    
    #[msg("Overflow in calculation")]
    Overflow,
    
    #[msg("Invalid multiplier value")]
    InvalidMultiplier,
    
    #[msg("Invalid fee structure")]
    InvalidFeeStructure,
} 