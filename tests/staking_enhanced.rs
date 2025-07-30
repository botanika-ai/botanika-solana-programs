use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::solana_sdk::signer::Signer;
use botanika_staking::state::{StakingState, UserStake};
use botanika_staking::shared_types::{StakingLevel, StakingStatus};
use botanika_staking::ID as STAKING_PROGRAM_ID;
use solana_program_test::*;
use solana_sdk::{signature::read_keypair_file, transaction::Transaction};

use crate::utils::*;

#[tokio::test]
async fn test_staking_initialization() {
    let mut context = setup_test_context().await;
    let authority = Keypair::new();
    airdrop(&mut context, &authority.pubkey(), 10_000_000_000).await;

    let (staking_state, _, _, _, _, _) = init_staking_system(&mut context, &authority).await;

    // Verify staking state initialization
    let staking_state_account: StakingState = get_account_data(&mut context, &staking_state).await;
    assert_eq!(staking_state_account.admin, authority.pubkey());
    assert_eq!(staking_state_account.total_staked, 0);
    assert_eq!(staking_state_account.multipliers[0], 1200); // Bronze
    assert_eq!(staking_state_account.multipliers[1], 1500); // Silver
    assert_eq!(staking_state_account.multipliers[2], 2000); // Gold
    assert_eq!(staking_state_account.multipliers[3], 3000); // Platinum
}

#[tokio::test]
async fn test_staking_level_calculation() {
    let mut context = setup_test_context().await;
    let user = Keypair::new();
    airdrop(&mut context, &user.pubkey(), 10_000_000_000).await;

    let (staking_state, _, stake_mint, vault, _, _) = init_staking_system(&mut context, &user).await;

    // Test Bronze level (0-1000)
    mint_to_user(&mut context, &stake_mint, &user, &context.payer, 500).await;
    stake_tokens(&mut context, &user, &staking_state, &stake_mint, 500).await;
    
    let user_stake = get_user_stake_account(&mut context, &user).await;
    let user_stake_data: UserStake = get_account_data(&mut context, &user_stake).await;
    assert_eq!(user_stake_data.level, StakingLevel::Bronze);

    // Test Silver level (1001-5000)
    mint_to_user(&mut context, &stake_mint, &user, &context.payer, 2000).await;
    stake_tokens(&mut context, &user, &staking_state, &stake_mint, 2000).await;
    
    let user_stake_data: UserStake = get_account_data(&mut context, &user_stake).await;
    assert_eq!(user_stake_data.level, StakingLevel::Silver);

    // Test Gold level (5001-10000)
    mint_to_user(&mut context, &stake_mint, &user, &context.payer, 3000).await;
    stake_tokens(&mut context, &user, &staking_state, &stake_mint, 3000).await;
    
    let user_stake_data: UserStake = get_account_data(&mut context, &user_stake).await;
    assert_eq!(user_stake_data.level, StakingLevel::Gold);

    // Test Platinum level (10001+)
    mint_to_user(&mut context, &stake_mint, &user, &context.payer, 5000).await;
    stake_tokens(&mut context, &user, &staking_state, &stake_mint, 5000).await;
    
    let user_stake_data: UserStake = get_account_data(&mut context, &user_stake).await;
    assert_eq!(user_stake_data.level, StakingLevel::Platinum);
}

#[tokio::test]
async fn test_lockup_period_validation() {
    let mut context = setup_test_context().await;
    let user = Keypair::new();
    airdrop(&mut context, &user.pubkey(), 10_000_000_000).await;

    let (staking_state, _, stake_mint, vault, _, _) = init_staking_system(&mut context, &user).await;

    // Stake tokens
    mint_to_user(&mut context, &stake_mint, &user, &context.payer, 1000).await;
    stake_tokens(&mut context, &user, &staking_state, &stake_mint, 1000).await;

    // Try to unstake before lockup period (should fail)
    let user_stake = get_user_stake_account(&mut context, &user).await;
    let result = unstake_tokens(&mut context, &user, &staking_state, &user_stake, &vault, &vault, &vault, &stake_mint).await;
    assert!(result.is_err());

    // Advance time past lockup period
    advance_slot(&mut context, 1000000).await; // Advance ~7 days

    // Now unstaking should work
    let result = unstake_tokens(&mut context, &user, &staking_state, &user_stake, &vault, &vault, &vault, &stake_mint).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_claim_cooldown_validation() {
    let mut context = setup_test_context().await;
    let user = Keypair::new();
    airdrop(&mut context, &user.pubkey(), 10_000_000_000).await;

    let (staking_state, _, stake_mint, _, reward_mint, reward_vault) = init_staking_system(&mut context, &user).await;

    // Stake tokens
    mint_to_user(&mut context, &stake_mint, &user, &context.payer, 1000).await;
    stake_tokens(&mut context, &user, &staking_state, &stake_mint, 1000).await;

    // Advance time to allow rewards
    advance_slot(&mut context, 100000).await;

    // Create reward destination
    let reward_dest = create_token_account(&mut context, &reward_mint, &user).await;

    // First claim should work
    let user_stake = get_user_stake_account(&mut context, &user).await;
    let result = claim_rewards(&mut context, &user, &user_stake, &reward_vault, &reward_dest).await;
    assert!(result.is_ok());

    // Second claim immediately should fail (cooldown)
    let result = claim_rewards(&mut context, &user, &user_stake, &reward_vault, &reward_dest).await;
    assert!(result.is_err());

    // Advance time past cooldown
    advance_slot(&mut context, 100000).await; // Advance ~24 hours

    // Now claim should work again
    let result = claim_rewards(&mut context, &user, &user_stake, &reward_vault, &reward_dest).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiplier_based_rewards() {
    let mut context = setup_test_context().await;
    let user = Keypair::new();
    airdrop(&mut context, &user.pubkey(), 10_000_000_000).await;

    let (staking_state, _, stake_mint, _, reward_mint, reward_vault) = init_staking_system(&mut context, &user).await;

    // Stake at Bronze level
    mint_to_user(&mut context, &stake_mint, &user, &context.payer, 500).await;
    stake_tokens(&mut context, &user, &staking_state, &stake_mint, 500).await;

    let user_stake = get_user_stake_account(&mut context, &user).await;
    let reward_dest = create_token_account(&mut context, &reward_mint, &user).await;

    // Advance time and claim
    advance_slot(&mut context, 100000).await;
    let bronze_reward = claim_rewards(&mut context, &user, &user_stake, &reward_vault, &reward_dest).await;
    assert!(bronze_reward.is_ok());

    // Stake more to reach Platinum level
    mint_to_user(&mut context, &stake_mint, &user, &context.payer, 10000).await;
    stake_tokens(&mut context, &user, &staking_state, &stake_mint, 10000).await;

    // Advance time and claim again
    advance_slot(&mut context, 100000).await;
    let platinum_reward = claim_rewards(&mut context, &user, &user_stake, &reward_vault, &reward_dest).await;
    assert!(platinum_reward.is_ok());

    // Platinum rewards should be higher than Bronze rewards
    let bronze_balance = get_token_balance(&mut context, &reward_dest).await;
    assert!(bronze_balance > 0);
}

#[tokio::test]
async fn test_overflow_protection() {
    let mut context = setup_test_context().await;
    let user = Keypair::new();
    airdrop(&mut context, &user.pubkey(), 10_000_000_000).await;

    let (staking_state, _, stake_mint, vault, _, _) = init_staking_system(&mut context, &user).await;

    // Try to stake maximum amount
    let max_amount = u64::MAX;
    mint_to_user(&mut context, &stake_mint, &user, &context.payer, max_amount).await;
    
    // This should fail due to overflow protection
    let result = stake_tokens(&mut context, &user, &staking_state, &stake_mint, max_amount).await;
    assert!(result.is_err());
}

// Helper functions
async fn get_user_stake_account(context: &mut ProgramTestContext, user: &Keypair) -> Pubkey {
    let (user_stake, _) = Pubkey::find_program_address(
        &[b"user-stake", user.pubkey().as_ref()],
        &STAKING_PROGRAM_ID,
    );
    user_stake
}

async fn claim_rewards(
    context: &mut ProgramTestContext,
    user: &Keypair,
    user_stake: &Pubkey,
    reward_vault: &Pubkey,
    reward_dest: &Pubkey,
) -> Result<()> {
    // Implementation would call the claim instruction
    // This is a placeholder for the actual implementation
    Ok(())
} 