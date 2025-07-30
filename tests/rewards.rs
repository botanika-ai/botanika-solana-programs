use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;
use anchor_lang::system_program;

use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use rewards::accounts as reward_accounts;
use rewards::instruction as reward_ix;

#[tokio::test]
async fn test_initialize_and_distribute_rewards() {
    // === Setup ===
    let program_id = rewards::ID;
    let mut program_test = ProgramTest::new(
        "rewards",
        program_id,
        processor!(rewards::entry),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Create accounts
    let vault = Keypair::new();
    let recipient = Keypair::new();

    // === Step 1: Initialize Vault ===
    let ix = reward_ix::initialize_vault(reward_accounts::InitializeVault {
        vault: vault.pubkey(),
        authority: payer.pubkey(),
        system_program: system_program::ID,
    });

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer, &vault],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // === Step 2: Airdrop Recipient and Create Account ===
    let rent = banks_client.get_rent().await.unwrap();
    let recipient_space = 8 + 8; // AccountDiscriminator + u64 rewards
    let lamports = rent.minimum_balance(recipient_space);

    let create_recipient = solana_sdk::system_instruction::create_account(
        &payer.pubkey(),
        &recipient.pubkey(),
        lamports,
        recipient_space as u64,
        &program_id,
    );

    let tx = Transaction::new_signed_with_payer(
        &[create_recipient],
        Some(&payer.pubkey()),
        &[&payer, &recipient],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // === Step 3: Distribute Rewards ===
    let distribute_ix = reward_ix::distribute(reward_accounts::Distribute {
        vault: vault.pubkey(),
        recipient: recipient.pubkey(),
        authority: payer.pubkey(),
    }, 1000);

    let tx = Transaction::new_signed_with_payer(
        &[distribute_ix],
        Some(&payer.pubkey()),
        &[&payer],
        banks_client.get_latest_blockhash().await.unwrap(),
    );
    banks_client.process_transaction(tx).await.unwrap();

    // === Step 4: Assert Recipient Reward Updated ===
    let recipient_account = banks_client
        .get_account(recipient.pubkey())
        .await
        .unwrap()
        .expect("recipient account not found");

    let recipient_data: rewards::RewardRecipient =
        Account::try_deserialize(&mut recipient_account.data.as_slice()).unwrap();

    assert_eq!(recipient_data.rewards_earned, 1000);
}
