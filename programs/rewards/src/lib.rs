use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

pub mod state;
pub mod error;

use botanika_common::*;
use crate::state::*;
use crate::error::RewardsError;

declare_id!("REwArds111111111111111111111111111111111111");

#[program]
pub mod rewards {
    use super::*;

    pub fn initialize_reward_pool(
        ctx: Context<InitializeRewardPool>,
        reward_mint: Pubkey,
        reward_vault: Pubkey,
        expiration_period: i64,
    ) -> Result<()> {
        let reward_pool = &mut ctx.accounts.reward_pool;
        reward_pool.authority = ctx.accounts.authority.key();
        reward_pool.reward_mint = reward_mint;
        reward_pool.reward_vault = reward_vault;
        reward_pool.expiration_period = expiration_period;
        reward_pool.total_rewards_distributed = 0;
        reward_pool.bump = *ctx.bumps.get("reward_pool").unwrap();
        Ok(())
    }

    pub fn submit_reward(
        ctx: Context<SubmitReward>,
        user: Pubkey,
        amount: u64,
        proof_type: ProofType,
    ) -> Result<()> {
        require!(amount > 0, RewardsError::InvalidAmount);
        
        let reward_pool = &mut ctx.accounts.reward_pool;
        let reward_recipient = &mut ctx.accounts.reward_recipient;
        
        // Update reward pool
        reward_pool.total_rewards_distributed = reward_pool.total_rewards_distributed
            .checked_add(amount)
            .ok_or(RewardsError::Overflow)?;
        
        // Initialize or update reward recipient
        reward_recipient.user = user;
        reward_recipient.proof_type = proof_type;
        reward_recipient.amount = amount;
        reward_recipient.status = RewardStatus::Available;
        reward_recipient.submitted_at = Clock::get()?.unix_timestamp;
        reward_recipient.expires_at = reward_recipient.submitted_at + reward_pool.expiration_period;
        reward_recipient.claimed_at = None;
        
        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let reward_recipient = &mut ctx.accounts.reward_recipient;
        let reward_pool = &ctx.accounts.reward_pool;
        
        require!(reward_recipient.can_claim(), RewardsError::RewardNotAvailable);
        require!(reward_recipient.user == ctx.accounts.user.key(), RewardsError::InvalidAuthority);
        
        // Transfer tokens to user
        token::transfer(ctx.accounts.into_transfer_ctx(), reward_recipient.amount)?;
        
        // Mark as claimed
        reward_recipient.mark_claimed();
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeRewardPool<'info> {
    #[account(init, payer = authority, space = 8 + RewardPool::SIZE, seeds = [b"reward-pool"], bump)]
    pub reward_pool: Account<'info, RewardPool>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SubmitReward<'info> {
    #[account(mut, seeds = [b"reward-pool"], bump = reward_pool.bump, has_one = authority)]
    pub reward_pool: Account<'info, RewardPool>,
    #[account(init_if_needed, payer = authority, space = 8 + RewardRecipient::SIZE, seeds = [b"reward-recipient", user.key().as_ref()], bump)]
    pub reward_recipient: Account<'info, RewardRecipient>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut, seeds = [b"reward-pool"], bump = reward_pool.bump)]
    pub reward_pool: Account<'info, RewardPool>,
    #[account(mut, seeds = [b"reward-recipient", user.key().as_ref()], bump)]
    pub reward_recipient: Account<'info, RewardRecipient>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub reward_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> ClaimReward<'info> {
    fn into_transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.reward_vault.to_account_info(),
            to: self.user_token_account.to_account_info(),
            authority: self.reward_pool.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}
