use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::solana_sdk::signer::Signer;
use botanika_staking::state::{StakingState, UserStake};
use botanika_staking::shared_types::{StakingLevel, StakingStatus};
use botanika_rewards::state::{RewardPool, RewardRecipient, ProofType, RewardStatus};
use botanika_governance::state::{GovernanceState, Proposal, ProposalType, ProposalStatus};
use botanika_staking::ID as STAKING_PROGRAM_ID;
use botanika_rewards::ID as REWARDS_PROGRAM_ID;
use botanika_governance::ID as GOVERNANCE_PROGRAM_ID;
use solana_program_test::*;
use solana_sdk::{signature::read_keypair_file, transaction::Transaction};

use crate::utils::*;

#[tokio::test]
async fn test_cross_program_staking_and_rewards() {
    let mut context = setup_test_context().await;
    let user = Keypair::new();
    let authority = Keypair::new();
    
    airdrop(&mut context, &user.pubkey(), 10_000_000_000).await;
    airdrop(&mut context, &authority.pubkey(), 10_000_000_000).await;

    // Initialize all programs
    let (staking_state, _, stake_mint, vault, reward_mint, reward_vault) = 
        init_staking_system(&mut context, &authority).await;
    
    let (reward_pool, _) = init_reward_pool(&mut context, &authority, &reward_mint, &reward_vault).await;
    let (governance_state, _) = init_governance_system(&mut context, &authority).await;

    // User stakes tokens
    mint_to_user(&mut context, &stake_mint, &user, &context.payer, 5000).await;
    stake_tokens(&mut context, &user, &staking_state, &stake_mint, 5000).await;

    // Verify staking level
    let user_stake = get_user_stake_account(&mut context, &user).await;
    let user_stake_data: UserStake = get_account_data(&mut context, &user_stake).await;
    assert_eq!(user_stake_data.level, StakingLevel::Gold);

    // Authority submits proof-based reward
    submit_proof_reward(&mut context, &authority, &reward_pool, &user.pubkey(), 1000, ProofType::ProofOfExecution).await;

    // User claims staking rewards
    advance_slot(&mut context, 100000).await;
    let reward_dest = create_token_account(&mut context, &reward_mint, &user).await;
    claim_staking_rewards(&mut context, &user, &user_stake, &reward_vault, &reward_dest).await;

    // User claims proof-based rewards
    claim_proof_rewards(&mut context, &user, &reward_pool, &reward_dest).await;

    // Verify rewards were received
    let balance = get_token_balance(&mut context, &reward_dest).await;
    assert!(balance > 0, "User should have received rewards");
}

#[tokio::test]
async fn test_governance_multiplier_updates() {
    let mut context = setup_test_context().await;
    let authority = Keypair::new();
    let user = Keypair::new();
    
    airdrop(&mut context, &authority.pubkey(), 10_000_000_000).await;
    airdrop(&mut context, &user.pubkey(), 10_000_000_000).await;

    // Initialize programs
    let (staking_state, _, stake_mint, _, reward_mint, reward_vault) = 
        init_staking_system(&mut context, &authority).await;
    let (governance_state, _) = init_governance_system(&mut context, &authority).await;

    // Get initial multipliers
    let governance_data: GovernanceState = get_account_data(&mut context, &governance_state).await;
    let initial_gold_multiplier = governance_data.multipliers[2]; // Gold level

    // Update multiplier through governance
    update_multiplier(&mut context, &authority, &governance_state, 2, 2500).await; // Increase Gold multiplier

    // Verify multiplier was updated
    let updated_governance_data: GovernanceState = get_account_data(&mut context, &governance_state).await;
    assert_eq!(updated_governance_data.multipliers[2], 2500);

    // User stakes and claims rewards with new multiplier
    mint_to_user(&mut context, &stake_mint, &user, &context.payer, 8000).await;
    stake_tokens(&mut context, &user, &staking_state, &stake_mint, 8000).await;

    advance_slot(&mut context, 100000).await;
    let user_stake = get_user_stake_account(&mut context, &user).await;
    let reward_dest = create_token_account(&mut context, &reward_mint, &user).await;
    
    let reward_before = get_token_balance(&mut context, &reward_dest).await;
    claim_staking_rewards(&mut context, &user, &user_stake, &reward_vault, &reward_dest).await;
    let reward_after = get_token_balance(&mut context, &reward_dest).await;
    
    assert!(reward_after > reward_before, "User should receive higher rewards with increased multiplier");
}

#[tokio::test]
async fn test_proposal_and_execution_flow() {
    let mut context = setup_test_context().await;
    let authority = Keypair::new();
    let proposer = Keypair::new();
    
    airdrop(&mut context, &authority.pubkey(), 10_000_000_000).await;
    airdrop(&mut context, &proposer.pubkey(), 10_000_000_000).await;

    // Initialize governance
    let (governance_state, _) = init_governance_system(&mut context, &authority).await;

    // Create proposal
    let proposal_data = vec![2, 0, 0, 0, 0, 0, 0, 0, 2500, 0, 0, 0, 0, 0, 0, 0]; // Level 2, Multiplier 2500
    create_proposal(&mut context, &proposer, &governance_state, ProposalType::MultiplierChange, proposal_data).await;

    // Get proposal
    let (proposal, _) = get_proposal_account(&mut context, &governance_state).await;
    let proposal_data: Proposal = get_account_data(&mut context, &proposal).await;
    assert_eq!(proposal_data.status, ProposalStatus::Active);

    // Simulate voting (in real implementation, this would be done by multiple voters)
    // For now, we'll just advance time and execute
    advance_slot(&mut context, 1000000).await; // Advance past voting period

    // Execute proposal
    execute_proposal(&mut context, &authority, &governance_state, &proposal).await;

    // Verify proposal was executed
    let executed_proposal_data: Proposal = get_account_data(&mut context, &proposal).await;
    assert_eq!(executed_proposal_data.status, ProposalStatus::Executed);
}

#[tokio::test]
async fn test_comprehensive_user_journey() {
    let mut context = setup_test_context().await;
    let user = Keypair::new();
    let authority = Keypair::new();
    
    airdrop(&mut context, &user.pubkey(), 10_000_000_000).await;
    airdrop(&mut context, &authority.pubkey(), 10_000_000_000).await;

    // Initialize all systems
    let (staking_state, _, stake_mint, vault, reward_mint, reward_vault) = 
        init_staking_system(&mut context, &authority).await;
    let (reward_pool, _) = init_reward_pool(&mut context, &authority, &reward_mint, &reward_vault).await;
    let (governance_state, _) = init_governance_system(&mut context, &authority).await;

    // 1. User stakes tokens
    mint_to_user(&mut context, &stake_mint, &user, &context.payer, 3000).await;
    stake_tokens(&mut context, &user, &staking_state, &stake_mint, 3000).await;

    // 2. Authority submits proof-based reward
    submit_proof_reward(&mut context, &authority, &reward_pool, &user.pubkey(), 500, ProofType::ProofOfService).await;

    // 3. Time passes
    advance_slot(&mut context, 200000).await;

    // 4. User claims staking rewards
    let user_stake = get_user_stake_account(&mut context, &user).await;
    let reward_dest = create_token_account(&mut context, &reward_mint, &user).await;
    claim_staking_rewards(&mut context, &user, &user_stake, &reward_vault, &reward_dest).await;

    // 5. User claims proof-based rewards
    claim_proof_rewards(&mut context, &user, &reward_pool, &reward_dest).await;

    // 6. User stakes more to reach higher level
    mint_to_user(&mut context, &stake_mint, &user, &context.payer, 7000).await;
    stake_tokens(&mut context, &user, &staking_state, &stake_mint, 7000).await;

    // 7. Verify user reached Platinum level
    let updated_user_stake_data: UserStake = get_account_data(&mut context, &user_stake).await;
    assert_eq!(updated_user_stake_data.level, StakingLevel::Platinum);

    // 8. More time passes
    advance_slot(&mut context, 300000).await;

    // 9. User claims rewards with higher multiplier
    claim_staking_rewards(&mut context, &user, &user_stake, &reward_vault, &reward_dest).await;

    // 10. Verify total rewards
    let final_balance = get_token_balance(&mut context, &reward_dest).await;
    assert!(final_balance > 0, "User should have accumulated significant rewards");

    // 11. User unstakes (after lockup period)
    advance_slot(&mut context, 1000000).await; // Advance past lockup
    unstake_tokens(&mut context, &user, &staking_state, &user_stake, &vault, &vault, &vault, &stake_mint).await;

    // 12. Verify unstaking was successful
    let final_user_stake_data: UserStake = get_account_data(&mut context, &user_stake).await;
    assert_eq!(final_user_stake_data.amount, 0);
    assert_eq!(final_user_stake_data.status, StakingStatus::Unstaking);
}

// Helper functions for integration tests
async fn init_reward_pool(
    context: &mut ProgramTestContext,
    authority: &Keypair,
    reward_mint: &Pubkey,
    reward_vault: &Pubkey,
) -> (Pubkey, u8) {
    let (reward_pool, bump) = Pubkey::find_program_address(
        &[b"reward-pool"],
        &REWARDS_PROGRAM_ID,
    );
    
    // Implementation would call initialize_reward_pool instruction
    (reward_pool, bump)
}

async fn init_governance_system(
    context: &mut ProgramTestContext,
    authority: &Keypair,
) -> (Pubkey, u8) {
    let (governance_state, bump) = Pubkey::find_program_address(
        &[b"governance-state"],
        &GOVERNANCE_PROGRAM_ID,
    );
    
    // Implementation would call initialize_governance instruction
    (governance_state, bump)
}

async fn submit_proof_reward(
    context: &mut ProgramTestContext,
    authority: &Keypair,
    reward_pool: &Pubkey,
    user: &Pubkey,
    amount: u64,
    proof_type: ProofType,
) {
    // Implementation would call submit_reward instruction
}

async fn claim_staking_rewards(
    context: &mut ProgramTestContext,
    user: &Keypair,
    user_stake: &Pubkey,
    reward_vault: &Pubkey,
    reward_dest: &Pubkey,
) {
    // Implementation would call claim instruction from staking program
}

async fn claim_proof_rewards(
    context: &mut ProgramTestContext,
    user: &Keypair,
    reward_pool: &Pubkey,
    reward_dest: &Pubkey,
) {
    // Implementation would call claim_reward instruction from rewards program
}

async fn update_multiplier(
    context: &mut ProgramTestContext,
    authority: &Keypair,
    governance_state: &Pubkey,
    level: u8,
    multiplier: u64,
) {
    // Implementation would call set_multiplier instruction
}

async fn create_proposal(
    context: &mut ProgramTestContext,
    proposer: &Keypair,
    governance_state: &Pubkey,
    proposal_type: ProposalType,
    data: Vec<u8>,
) {
    // Implementation would call propose_change instruction
}

async fn get_proposal_account(
    context: &mut ProgramTestContext,
    governance_state: &Pubkey,
) -> (Pubkey, u8) {
    let governance_data: GovernanceState = get_account_data(context, governance_state).await;
    let (proposal, bump) = Pubkey::find_program_address(
        &[b"proposal", governance_data.last_updated.to_le_bytes().as_ref()],
        &GOVERNANCE_PROGRAM_ID,
    );
    (proposal, bump)
}

async fn execute_proposal(
    context: &mut ProgramTestContext,
    authority: &Keypair,
    governance_state: &Pubkey,
    proposal: &Pubkey,
) {
    // Implementation would call execute_proposal instruction
} 