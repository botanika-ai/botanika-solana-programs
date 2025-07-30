use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, Transfer};

use crate::state::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 32 + 1, seeds = [b"state"], bump)]
    pub state: Account<'info, State>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// vault must be a token account owned by state
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 8 + 8,
        seeds = [b"stake", user.key().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, StakeAccount>,

    #[account(mut)]
    pub user_token: Account<'info, TokenAccount>,

    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> Stake<'info> {
    pub fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.user_token.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.user.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"stake", user.key().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, StakeAccount>,

    #[account(
        seeds = [b"state"],
        bump = state.bump
    )]
    pub state: Account<'info, State>,

    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

impl<'info> Claim<'info> {
    pub fn reward_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let seeds = &[b"state", &[self.state.bump]];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user_token.to_account_info(),
            authority: self.state.to_account_info(),
        };
        CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_accounts, signer)
    }
}
