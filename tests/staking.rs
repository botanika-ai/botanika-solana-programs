use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::solana_sdk::signer::Signer;
use botanika_staking::state::{StakingState, UserStake};
use botanika_staking::ID as STAKING_PROGRAM_ID;
use solana_program_test::*;
use solana_sdk::{signature::read_keypair_file, transaction::Transaction};

use crate::utils::*;

#[tokio::test]
async fn test_unstake_with_auto_claim() {
    let mut context = setup_test_context().await;

    // Create user
    let user = Keypair::new();
    airdrop(&mut context, &user.pubkey(), 10_000_000_000).await;

    // Initialize staking
    let (staking_state, user_stake, stake_mint, vault, reward_mint, reward_vault) =
        init_staking_system(&mut context, &user).await;

    // Mint some stake tokens to user
    mint_to_user(
        &mut context,
        &stake_mint,
        &user,
        &context.payer,
        1_000_000_000,
    )
    .await;

    // Approve and stake tokens
    stake_tokens(
        &mut context,
        &user,
        &staking_state,
        &stake_mint,
        1_000_000_000,
    )
    .await;

    // Simulate passage of time
    advance_slot(&mut context, 100).await;

    // Create reward destination account
    let reward_dest = create_token_account(&mut context, &reward_mint, &user).await;

    // Call unstake (should include auto-claim)
    unstake_tokens(
        &mut context,
        &user,
        &staking_state,
        &user_stake,
        &vault,
        &reward_vault,
        &reward_dest,
        &stake_mint,
    )
    .await;

    // Verify reward balance
    let reward_balance = get_token_balance(&mut context, &reward_dest).await;
    assert!(reward_balance > 0, "User should have received reward");

    // Verify stake amount is 0
    let user_stake_account: UserStake = get_account_data(&mut context, &user_stake).await;
    assert_eq!(user_stake_account.amount, 0);
}
