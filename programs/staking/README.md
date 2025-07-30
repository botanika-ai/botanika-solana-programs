# Botanika - Staking Program

This program handles staking functionality for the Botanika network using the BONSAI token.

## Features

- Initialize staking config with reward rate
- Stake tokens (tracked by user)
- Claim rewards based on elapsed time and stake amount

## Instructions

- `initialize(reward_rate: u64)`  
  Initializes the staking config. `reward_rate` is the per-second reward per lamport staked.

- `stake(amount: u64)`  
  Stake the given amount of BONSAI tokens. Creates or updates a stake account.

- `claim()`  
  Claim the accumulated rewards. Rewards are calculated using:  
  `reward = elapsed_time * reward_rate * staked_amount / 100`

## Accounts

- `Config`: Global config for staking parameters.
- `StakeInfo`: User-specific stake tracking account.
- `RewardVault`: Account holding reward lamports (must be pre-funded).

## Usage Flow

1. `initialize` → 2. `stake` → 3. Wait → 4. `claim`

---

## Dev Setup

```bash
anchor build
anchor test
```