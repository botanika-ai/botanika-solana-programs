# Rewards Program

This program handles BONSAI staking reward distribution logic for the Botanika network.

## Features

- Distribute rewards based on staking amount and duration.
- Configurable reward vault controlled by PDA.
- Supports claim cycles and authorized reward injection.

## Flow

1. `initialize_rewards`: Create vault and authority.
2. `inject_rewards`: Admin-only injection of funds.
3. `claim_rewards`: User claims available rewards.
4. `close_rewards`: Admin disables reward flow.

## PDA Accounts

- Reward Vault PDA
- Authority PDA

## Dependencies

- `anchor-lang`
- `anchor-spl`

