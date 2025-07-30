# Governance Program

The governance program supports basic parameter and authority management for Botanika smart contracts.

## Features

- Admin assignment & transfer
- Global config updates
- Staking/reward rate changes (if delegated)

## Flow

1. `initialize_governance`: Creates config with current admin.
2. `set_admin`: Only callable by current admin.
3. `update_config`: Update critical params like reward rate.

## PDA Accounts

- Governance Config PDA

## Security

- Access control strictly enforced via signer check and PDA seeds.
- Modifiers used to validate authorities before state changes.

## Dependencies

- `anchor-lang`
- `anchor-spl`
