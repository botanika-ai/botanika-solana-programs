use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    pubkey::Pubkey,
    system_instruction,
};

/// Creates a new SPL mint with specified decimals and mint authority.
pub async fn create_mint(
    context: &mut ProgramTestContext,
    mint_authority: &Pubkey,
    decimals: u8,
) -> Result<Pubkey, TransportError> {
    let mint = Keypair::new();
    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(Mint::LEN);

    let create_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &mint.pubkey(),
        lamports,
        Mint::LEN as u64,
        &token::ID,
    );

    let init_ix = token::instruction::initialize_mint(
        &token::ID,
        &mint.pubkey(),
        mint_authority,
        None,
        decimals,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, init_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await?;
    Ok(mint.pubkey())
}

/// Creates a new token account owned by the given wallet for the specified mint.
pub async fn create_token_account(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    owner: &Pubkey,
) -> Result<Pubkey, TransportError> {
    let token_account = Keypair::new();
    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(TokenAccount::LEN);

    let create_ix = system_instruction::create_account(
        &context.payer.pubkey(),
        &token_account.pubkey(),
        lamports,
        TokenAccount::LEN as u64,
        &token::ID,
    );

    let init_ix = token::instruction::initialize_account(
        &token::ID,
        &token_account.pubkey(),
        mint,
        owner,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, init_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &token_account],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await?;
    Ok(token_account.pubkey())
}

/// Mints tokens to the specified account using the mint authority.
pub async fn mint_tokens(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    to: &Pubkey,
    amount: u64,
    authority: &Keypair,
) -> Result<(), TransportError> {
    let ix = token::instruction::mint_to(
        &token::ID,
        mint,
        to,
        &authority.pubkey(),
        &[],
        amount,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

// Get token balance
pub async fn get_token_balance(
    context: &mut ProgramTestContext,
    account: &Pubkey,
) -> u64 {
    let acc = context
        .banks_client
        .get_account(*account)
        .await
        .unwrap()
        .unwrap();
    let token_acc = TokenAccount::try_deserialize(&mut &acc.data[..]).unwrap();
    token_acc.amount
}

// Advance slot to simulate time
pub async fn advance_slot(
    context: &mut ProgramTestContext,
    slot_count: u64,
) {
    context.warp_to_slot(context.slot + slot_count).unwrap();
}

// Placeholder for stake
pub async fn stake_tokens(
    context: &mut ProgramTestContext,
    user: &Keypair,
    staking_state: &Pubkey,
    stake_mint: &Pubkey,
    amount: u64,
) {
    // You'd typically call the program’s CPI here
    // This is just a placeholder
    println!("Simulate staking {} tokens", amount);
}

// Placeholder for unstake
pub async fn unstake_tokens(
    context: &mut ProgramTestContext,
    user: &Keypair,
    staking_state: &Pubkey,
    user_stake: &Pubkey,
    vault: &Pubkey,
    reward_vault: &Pubkey,
    reward_dest: &Pubkey,
    stake_mint: &Pubkey,
) {
    // You'd typically call the program’s CPI here
    println!("Simulate unstaking and reward claim");
}

// Initialize full staking system (simplified placeholder)
pub async fn init_staking_system(
    context: &mut ProgramTestContext,
    user: &Keypair,
) -> (Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey) {
    let stake_mint = create_mint(context, &context.payer.pubkey()).await;
    let reward_mint = create_mint(context, &context.payer.pubkey()).await;

    let staking_state = Pubkey::new_unique();
    let user_stake = Pubkey::new_unique();
    let vault = Pubkey::new_unique();
    let reward_vault = Pubkey::new_unique();

    (
        staking_state,
        user_stake,
        stake_mint,
        vault,
        reward_mint,
        reward_vault,
    )
}