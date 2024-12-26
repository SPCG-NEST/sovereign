use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use mpl_core::{instructions::*, ID as MPL_CORE_ID};

use crate::{
    constant::{
        BALANCES_INIT, BROKER_ESCROW_SEED, GAME_SEED, POOL_SEED, TXN_FEE, WALLET_SEED, WEIGHTS_INIT,
    },
    error::SovereignError,
    state::{BrokerEscrow, Game, Pool, Wallet},
};

pub fn init_game(ctx: Context<InitGame>, game_id: u64, init_game_args: InitGameArgs) -> Result<()> {
    let system_program = ctx.accounts.system_program.to_account_info();
    let world_agent_wallet = ctx.accounts.world_agent_wallet.to_account_info();

    let payer = ctx.accounts.payer.to_account_info();
    let authority = payer.key();
    create_collection(&ctx, game_id, &init_game_args)?;
    let mut ctx = ctx;
    initialize_game(&mut ctx, game_id, init_game_args)?;

    // Transfer sol
    let transfer_accounts = Transfer {
        from: payer,
        to: world_agent_wallet,
    };

    transfer(
        CpiContext::new(system_program, transfer_accounts),
        TXN_FEE
            .checked_mul(25_000)
            .ok_or(SovereignError::MathOverflow)?,
    )?;

    // Log game event
    log_game_event(game_id, authority)?;

    Ok(())
}

#[inline(never)]
fn log_game_event(game_id: u64, authority: Pubkey) -> Result<()> {
    emit!(NewGameEvent {
        game_id,
        authority: authority.to_string(),
    });
    Ok(())
}

#[inline(never)]
fn create_collection(
    ctx: &Context<InitGame>,
    game_id: u64,
    init_game_args: &InitGameArgs,
) -> Result<()> {
    // First, prepare all the data we need to copy
    let game_id = game_id;

    // Create collection first
    let signers_seeds = &[
        GAME_SEED.as_bytes(),
        &game_id.to_le_bytes(),
        &[ctx.bumps.game_account],
    ];

    CreateCollectionV2CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .collection(&ctx.accounts.collection.to_account_info())
        .payer(&ctx.accounts.payer.to_account_info())
        .update_authority(Some(&ctx.accounts.game_account.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .name(format!("Soverign {:#}", game_id).to_string())
        .uri(init_game_args.collection_uri.to_string())
        .invoke_signed(&[signers_seeds])?;

    Ok(())
}

#[inline(never)]
fn initialize_game(
    ctx: &mut Context<InitGame>,
    game_id: u64,
    init_game_args: InitGameArgs,
) -> Result<()> {
    // Initialize game account
    ctx.accounts.game_account.set_inner(Game {
        id: game_id,
        authority: ctx.accounts.payer.key(),
        collection: ctx.accounts.collection.key(),
        slot_start: init_game_args.slot_start,
        world_agent: init_game_args.world_agent,
        broker_key: init_game_args.broker_key,
        mint_cost: init_game_args.mint_cost,
        bounty_pow_threshold: [0u8; 32],
        nations_alive: 0,
        citizen_stake_length: 2 * 60 * 60 * 6, // ~2 slots/s * 60s/m * 60m/h * 6h
    });

    // Initialize broker escrow
    ctx.accounts.broker_escrow.game_id = game_id;
    ctx.accounts.broker_escrow.broker_key = init_game_args.broker_key;

    // Initialize pool
    ctx.accounts.game_pool.game_id = game_id;
    ctx.accounts.game_pool.balances = BALANCES_INIT;
    ctx.accounts.game_pool.weights = WEIGHTS_INIT;

    // Initialize world agent wallet
    ctx.accounts.world_agent_wallet.game_id = game_id;
    ctx.accounts.world_agent_wallet.authority = init_game_args.world_agent;
    ctx.accounts.world_agent_wallet.balances = BALANCES_INIT;
    Ok(())
}

#[event]
pub struct NewGameEvent {
    pub game_id: u64,
    pub authority: String,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitGameArgs {
    pub slot_start: u64,
    pub collection_uri: String,
    pub world_agent: Pubkey,
    pub broker_key: Pubkey,
    pub mint_cost: u64,
    pub bounty_pow_threshold: [u8; 32],
}

#[derive(Accounts)]
#[instruction(game_id: u64, init_game_args: InitGameArgs)]
pub struct InitGame<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account (
        init,
        space = 8 + Game::INIT_SPACE,
        payer = payer,
        seeds = [GAME_SEED.as_bytes().as_ref(), &game_id.to_le_bytes().as_ref()],
        bump
    )]
    pub game_account: Box<Account<'info, Game>>,
    #[account(
        init,
        space = 8 + Wallet::INIT_SPACE,
        payer = payer,
        seeds = [WALLET_SEED.as_bytes().as_ref(), &game_id.to_le_bytes().as_ref(), &init_game_args.world_agent.to_bytes().as_ref()],
        bump
    )]
    pub world_agent_wallet: Box<Account<'info, Wallet>>,
    #[account(
        init,
        space = 8 + Pool::INIT_SPACE,
        payer = payer,
        seeds = [POOL_SEED.as_bytes().as_ref(),  &game_id.to_le_bytes().as_ref()],
        bump
    )]
    pub game_pool: Box<Account<'info, Pool>>,
    #[account(
        init,
        space = 8 + BrokerEscrow::INIT_SPACE,
        payer = payer,
        seeds = [BROKER_ESCROW_SEED.as_bytes().as_ref(), &game_id.to_le_bytes().as_ref()],
        bump
    )]
    pub broker_escrow: Box<Account<'info, BrokerEscrow>>,
    ///CHECK: Created by CPI
    #[account(mut)]
    pub collection: Signer<'info>,
    /// CHECK: this account is checked by the address constraint
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
