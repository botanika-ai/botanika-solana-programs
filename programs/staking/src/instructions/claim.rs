use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, Transfer, transfer};
use crate::state::{StakingState, UserStake};

#[derive(Accounts)]
pub struct Claim<'info> {
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
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(
        mut,
        token::mint = reward_mint,
        token::authority = staking_state,
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub reward_destination: Account<'info, TokenAccount>,

    pub reward_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(ctx: Context<Claim>) -> Result<()> {
    let staking_state = &mut ctx.accounts.staking_state;
    let user_stake = &mut ctx.accounts.user_stake;
    let current_slot = ctx.accounts.clock.slot;

    // Calculate reward: (current_slot - last_updated) * reward_rate * user.amount * multiplier
    let elapsed_slots = current_slot
        .checked_sub(user_stake.last_updated_slot)
        .ok_or_else(|| error!(ErrorCode::Overflow))?;

    if elapsed_slots == 0 {
        return Ok(());
    }

    let base_reward = elapsed_slots
        .checked_mul(staking_state.reward_rate_per_slot)
        .and_then(|r| r.checked_mul(user_stake.amount))
        .ok_or_else(|| error!(ErrorCode::Overflow))?;

    let multiplier = u64::from(user_stake.multiplier.max(1));
    let final_reward = base_reward
        .checked_mul(multiplier)
        .ok_or_else(|| error!(ErrorCode::Overflow))?;

    // Send reward tokens
    let seeds = &[b"staking_state", &[staking_state.bump]];
    let signer_seeds = &[&seeds[..]];

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.reward_destination.to_account_info(),
                authority: staking_state.to_account_info(),
            },
            signer_seeds,
        ),
        final_reward,
    )?;

    // Update state
    user_stake.reward_debt = 0;
    user_stake.last_updated_slot = current_slot;

    Ok(())
}
