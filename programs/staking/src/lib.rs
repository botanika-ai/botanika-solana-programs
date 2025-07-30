/// File: programs/staking/src/lib.rs
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

pub mod errors;
pub mod state;
use botanika_common::*;

use crate::errors::StakingError;
use crate::state::*;

declare_id!("Stake11111111111111111111111111111111111111");

#[program]
pub mod staking {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let staking_state = &mut ctx.accounts.staking_state;
        staking_state.admin = ctx.accounts.authority.key();
        staking_state.bump = *ctx.bumps.get("staking_state").unwrap();
        staking_state.total_staked = 0;
        staking_state.multipliers = [BRONZE_MULTIPLIER, SILVER_MULTIPLIER, GOLD_MULTIPLIER, PLATINUM_MULTIPLIER];
        staking_state.lockup_period = LOCKUP_PERIOD;
        staking_state.claim_cooldown = CLAIM_COOLDOWN;
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        require!(amount > 0, StakingError::InvalidAmount);
        let staking_state = &mut ctx.accounts.staking_state;
        let user_stake = &mut ctx.accounts.user_stake;

        // Transfer tokens to vault
        token::transfer(ctx.accounts.into_transfer_to_vault_ctx(), amount)?;

        // Update staking state
        staking_state.total_staked = staking_state.total_staked.checked_add(amount)
            .ok_or(StakingError::Overflow)?;
        
        user_stake.owner = ctx.accounts.user.key();
        user_stake.amount = user_stake.amount.checked_add(amount)
            .ok_or(StakingError::Overflow)?;
        user_stake.staked_at = Clock::get()?.unix_timestamp;
        user_stake.lockup_end = user_stake.staked_at + staking_state.lockup_period;
        user_stake.level = user_stake.calculate_level();
        user_stake.status = StakingStatus::Active;
        user_stake.last_claimed_at = 0;
        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        require!(amount > 0, StakingError::InvalidAmount);
        let staking_state = &mut ctx.accounts.staking_state;
        let user_stake = &mut ctx.accounts.user_stake;
        require!(user_stake.amount >= amount, StakingError::InsufficientStake);
        require!(user_stake.is_lockup_met(), StakingError::LockupPeriodNotMet);

        // Transfer tokens back to user
        token::transfer(ctx.accounts.into_transfer_to_user_ctx(), amount)?;

        staking_state.total_staked = staking_state.total_staked.checked_sub(amount)
            .ok_or(StakingError::Overflow)?;
        user_stake.amount = user_stake.amount.checked_sub(amount)
            .ok_or(StakingError::Overflow)?;
        
        // Update level if amount changed significantly
        user_stake.level = user_stake.calculate_level();
        
        // If unstaking everything, update status
        if user_stake.amount == 0 {
            user_stake.status = StakingStatus::Unstaking;
        }
        Ok(())
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let user_stake = &mut ctx.accounts.user_stake;
        require!(user_stake.can_claim(), StakingError::ClaimCooldownNotMet);

        let now = Clock::get()?.unix_timestamp;
        let time_since_last_claim = now.checked_sub(user_stake.last_claimed_at)
            .ok_or(StakingError::Overflow)?;

        // Calculate reward based on amount, time, and multiplier
        let multiplier = user_stake.get_multiplier(&ctx.accounts.staking_state);
        let base_reward = user_stake.amount.checked_mul(time_since_last_claim)
            .ok_or(StakingError::Overflow)?;
        let reward = base_reward.checked_mul(multiplier)
            .ok_or(StakingError::Overflow)?
            .checked_div(1000)
            .ok_or(StakingError::Overflow)?
            .checked_div(365 * 24 * 60 * 60) // Annual rate
            .ok_or(StakingError::Overflow)?;

        require!(reward > 0, StakingError::InvalidAmount);

        token::transfer(ctx.accounts.into_reward_transfer_ctx(), reward)?;
        user_stake.last_claimed_at = now;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + StakingState::SIZE, seeds = [b"staking-state"], bump)]
    pub staking_state: Account<'info, StakingState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub staking_state: Account<'info, StakingState>,
    #[account(init_if_needed, payer = user, space = 8 + UserStake::SIZE, seeds = [b"user-stake", user.key().as_ref()], bump)]
    pub user_stake: Account<'info, UserStake>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub staking_state: Account<'info, StakingState>,
    #[account(mut, seeds = [b"user-stake", user.key().as_ref()], bump)]
    pub user_stake: Account<'info, UserStake>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut, seeds = [b"user-stake", user.key().as_ref()], bump)]
    pub user_stake: Account<'info, UserStake>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub reward_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Stake<'info> {
    fn into_transfer_to_vault_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.user_token_account.to_account_info(),
            to: self.vault_token_account.to_account_info(),
            authority: self.user.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

impl<'info> Unstake<'info> {
    fn into_transfer_to_user_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault_token_account.to_account_info(),
            to: self.user_token_account.to_account_info(),
            authority: self.staking_state.to_account_info(), // will be signed by PDA in future
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

impl<'info> Claim<'info> {
    fn into_reward_transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.reward_vault.to_account_info(),
            to: self.user_token_account.to_account_info(),
            authority: self.reward_vault.to_account_info(), // change to reward authority PDA
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}
