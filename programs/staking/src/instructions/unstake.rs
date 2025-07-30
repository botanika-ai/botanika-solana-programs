use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Transfer, transfer};
use crate::state::{StakingState, UserStake};
use crate::ErrorCode;

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"staking_state"],
        bump = staking_state.bump,
    )]
    pub staking_state: Account<'info, StakingState>,

    #[account(
        mut,
        seeds = [b"user_stake", user.key().as_ref()],
        bump = user_stake.bump,
        constraint = user_stake.amount > 0 @ ErrorCode::NothingToUnstake,
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(
        mut,
        token::authority = staking_state,
        token::mint = staking_state.stake_mint,
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = staking_state.reward_mint,
        token::authority = staking_state,
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub reward_destination: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<Unstake>) -> Result<()> {
    let staking_state = &mut ctx.accounts.staking_state;
    let user_stake = &mut ctx.accounts.user_stake;
    let current_slot = ctx.accounts.clock.slot;

    // Step 1: Calculate automatic reward (claim built-in)
    let elapsed_slots = current_slot
        .checked_sub(user_stake.last_updated_slot)
        .ok_or(ErrorCode::Overflow)?;

    let base_reward = elapsed_slots
        .checked_mul(staking_state.reward_rate_per_slot)
        .and_then(|r| r.checked_mul(user_stake.amount))
        .ok_or(ErrorCode::Overflow)?;

    let multiplier = u64::from(user_stake.multiplier.max(1));
    let final_reward = base_reward
        .checked_mul(multiplier)
        .ok_or(ErrorCode::Overflow)?;

    let signer_seeds = &[b"staking_state", &[staking_state.bump]];
    let signer = &[&signer_seeds[..]];

    if final_reward > 0 {
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.reward_vault.to_account_info(),
                    to: ctx.accounts.reward_destination.to_account_info(),
                    authority: staking_state.to_account_info(),
                },
                signer,
            ),
            final_reward,
        )?;
    }

    // Step 2: Return unstaked amount
    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: staking_state.to_account_info(),
            },
            signer,
        ),
        user_stake.amount,
    )?;

    // Step 3: Initialize state
    user_stake.amount = 0;
    user_stake.last_updated_slot = current_slot;
    user_stake.reward_debt = 0;

    Ok(())
}
