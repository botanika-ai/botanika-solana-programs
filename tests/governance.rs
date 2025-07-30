use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;
use anchor_lang::system_program;

use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use governance::instruction as gov_ix;
use governance::accounts as gov_accounts;

#[tokio::test]
async fn test_initialize_and_update_config() {
    let program_id = governance::ID;

    let mut program_test = ProgramTest::new(
        "governance",
        program_id,
        processor!(governance::entry),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // === Step 1: Initialize Governance Config ===
    let config_account = Keypair::new();

    let ix = gov_ix::initialize_config(gov_accounts::InitializeConfig {
        config: config_account.pubkey(),
        authority: payer.pubkey(),
        system_program: system_program::ID,
    });

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer, &config_account],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await.unwrap();

    // === Step 2: Update Governance Quorum ===
    let update_ix = gov_ix::update_quorum(
        gov_accounts::UpdateQuorum {
            config: config_account.pubkey(),
            authority: payer.pubkey(),
        },
        75, // new quorum value
    );

    let tx = Transaction::new_signed_with_payer(
        &[update_ix],
        Some(&payer.pubkey()),
        &[&payer],
        banks_client.get_latest_blockhash().await.unwrap(),
    );
    banks_client.process_transaction(tx).await.unwrap();

    // === Step 3: Validate Updated State ===
    let config_data_account = banks_client
        .get_account(config_account.pubkey())
        .await
        .unwrap()
        .expect("Config account not found");

    let config_data: governance::GovernanceConfig =
        Account::try_deserialize(&mut config_data_account.data.as_slice()).unwrap();

    assert_eq!(config_data.quorum, 75);
}
