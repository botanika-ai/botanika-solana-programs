use anchor_lang::prelude::*;

pub mod state;
pub mod error;

use botanika_common::*;
use crate::state::*;
use crate::error::GovernanceError;

declare_id!("Govrnce1111111111111111111111111111111111");

#[program]
pub mod botanika_governance {
    use super::*;

    pub fn initialize_governance(ctx: Context<InitializeGovernance>) -> Result<()> {
        let governance_state = &mut ctx.accounts.governance_state;
        governance_state.authority = ctx.accounts.authority.key();
        governance_state.multipliers = [1200, 1500, 2000, 3000]; // Bronze, Silver, Gold, Platinum
        governance_state.staking_fee_bps = 50;    // 0.5%
        governance_state.unstaking_fee_bps = 100; // 1.0%
        governance_state.claim_fee_bps = 25;      // 0.25%
        governance_state.proposal_duration = 3 * 24 * 60 * 60; // 3 days
        governance_state.quorum_threshold = 1000; // Minimum votes
        governance_state.last_updated = Clock::get()?.unix_timestamp;
        governance_state.bump = *ctx.bumps.get("governance_state").unwrap();
        Ok(())
    }

    pub fn set_multiplier(
        ctx: Context<SetMultiplier>,
        level: u8,
        multiplier: u64,
    ) -> Result<()> {
        require!(level < 4, GovernanceError::InvalidMultiplier);
        require!(multiplier > 0, GovernanceError::InvalidMultiplier);
        
        let governance_state = &mut ctx.accounts.governance_state;
        governance_state.multipliers[level as usize] = multiplier;
        governance_state.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn propose_change(
        ctx: Context<ProposeChange>,
        proposal_type: ProposalType,
        data: Vec<u8>,
    ) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let governance_state = &ctx.accounts.governance_state;
        
        proposal.id = governance_state.last_updated; // Simple ID generation
        proposal.proposal_type = proposal_type;
        proposal.status = ProposalStatus::Active;
        proposal.data = data;
        proposal.created_at = Clock::get()?.unix_timestamp;
        proposal.voting_ends_at = proposal.created_at + governance_state.proposal_duration;
        proposal.votes_for = 0;
        proposal.votes_against = 0;
        proposal.bump = *ctx.bumps.get("proposal").unwrap();
        
        Ok(())
    }

    pub fn execute_proposal(ctx: Context<ExecuteProposal>) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let governance_state = &mut ctx.accounts.governance_state;
        
        require!(proposal.is_active(), GovernanceError::ProposalNotActive);
        require!(proposal.voting_ended(), GovernanceError::VotingPeriodNotEnded);
        require!(proposal.passed(), GovernanceError::InsufficientVotingPower);
        
        // Execute the proposal based on type
        match proposal.proposal_type {
            ProposalType::MultiplierChange => {
                // Parse and apply multiplier changes
                // Implementation would depend on data format
            },
            ProposalType::FeeStructure => {
                // Parse and apply fee structure changes
                // Implementation would depend on data format
            },
            _ => {
                // Handle other proposal types
            }
        }
        
        proposal.mark_executed();
        governance_state.last_updated = Clock::get()?.unix_timestamp;
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeGovernance<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + GovernanceState::SIZE,
        seeds = [b"governance-state"],
        bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetMultiplier<'info> {
    #[account(mut, seeds = [b"governance-state"], bump = governance_state.bump, has_one = authority)]
    pub governance_state: Account<'info, GovernanceState>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ProposeChange<'info> {
    #[account(seeds = [b"governance-state"], bump = governance_state.bump)]
    pub governance_state: Account<'info, GovernanceState>,
    #[account(init, payer = proposer, space = 8 + Proposal::SIZE, seeds = [b"proposal", governance_state.last_updated.to_le_bytes().as_ref()], bump)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub proposer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    #[account(mut, seeds = [b"governance-state"], bump = governance_state.bump, has_one = authority)]
    pub governance_state: Account<'info, GovernanceState>,
    #[account(mut, seeds = [b"proposal", governance_state.last_updated.to_le_bytes().as_ref()], bump)]
    pub proposal: Account<'info, Proposal>,
    pub authority: Signer<'info>,
}
