use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};

use crate::{constant::{BROKER_ESCROW_SEED, GAME_SEED, NATION_SEED, TXN_FEE, WALLET_SEED}, error::SovereignError, state::{BrokerEscrow, Game, Nation, Wallet}};

pub fn init_nation(ctx: Context<InitNation>, init_nation_args: InitNationArgs) -> Result<()> {

    ctx.accounts.nation.game_id = ctx.accounts.game.id;
    ctx.accounts.nation.nation_id = init_nation_args.id;
    ctx.accounts.nation.authority = ctx.accounts.nation_authority.key();
    ctx.accounts.nation.gdp = init_nation_args.gdp;
    ctx.accounts.nation.healthcare = init_nation_args.healthcare;
    ctx.accounts.nation.environment = init_nation_args.environment;
    ctx.accounts.nation.stability = init_nation_args.stability;

    ctx.accounts.nation.gdp_reward_rate = init_nation_args.gdp_reward_rate;
    ctx.accounts.nation.healthcare_reward_rate = init_nation_args.healthcare_reward_rate;
    ctx.accounts.nation.environment_reward_rate = init_nation_args.environment_reward_rate;
    ctx.accounts.nation.stability_reward_rate = init_nation_args.stability_reward_rate;

    ctx.accounts.nation.is_alive = true;
    ctx.accounts.game.nations_alive += 1;
    ctx.accounts.nation.minted_tokens_total = 0;

    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitNationArgs {
    pub id: u8,
    pub gdp: u64,
    pub healthcare: u64,
    pub environment: u64,
    pub stability: u64,
    pub gdp_reward_rate: u64,
    pub healthcare_reward_rate: u64,
    pub environment_reward_rate: u64,
    pub stability_reward_rate: u64
}

#[derive(Accounts)]
#[instruction(init_nation_args: InitNationArgs)]
pub struct InitNation<'info> {
    #[account(mut)]
    pub game_authority: Signer<'info>,
    pub nation_authority: Signer<'info>,
    #[account(mut,
        constraint = game.authority == game_authority.key() @ SovereignError::InvalidAuthority
    )]
    pub game: Account<'info, Game>,
    #[account(
        init,
        payer = game_authority,
        space = 8 + Nation::INIT_SPACE,
        seeds = [NATION_SEED.as_bytes(), game.id.to_le_bytes().as_ref(), init_nation_args.id.to_le_bytes().as_ref()],
        bump
    )]
    pub nation: Account<'info, Nation>,
    pub system_program: Program<'info, System>,
}

pub fn update_nation_reward_rate(ctx: Context<UpdateNationRewardRate>, update_nation_reward_rate_args: UpdateNationRewardRateArgs) -> Result<()> {
    require!(ctx.accounts.nation.is_alive, SovereignError::NationIsDead);

    ctx.accounts.nation.gdp_reward_rate = update_nation_reward_rate_args.gdp_reward_rate;
    ctx.accounts.nation.healthcare_reward_rate = update_nation_reward_rate_args.healthcare_reward_rate;
    ctx.accounts.nation.environment_reward_rate = update_nation_reward_rate_args.environment_reward_rate;
    ctx.accounts.nation.stability_reward_rate = update_nation_reward_rate_args.stability_reward_rate;

    emit!(UpdateNationRewardRateEvent {
        game_id: ctx.accounts.nation.game_id,
        nation_id: ctx.accounts.nation.nation_id,
        gdp_reward_rate: update_nation_reward_rate_args.gdp_reward_rate,
        healthcare_reward_rate: update_nation_reward_rate_args.healthcare_reward_rate,
        environment_reward_rate: update_nation_reward_rate_args.environment_reward_rate,
        stability_reward_rate: update_nation_reward_rate_args.stability_reward_rate,
        slot: Clock::get()?.slot,
    });

    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateNationRewardRateArgs {
    pub gdp_reward_rate: u64,
    pub healthcare_reward_rate: u64,
    pub environment_reward_rate: u64,
    pub stability_reward_rate: u64
}

#[derive(Accounts)]
pub struct UpdateNationRewardRate<'info> {
    pub nation_authority: Signer<'info>,
    #[account(
        mut,    
        constraint = nation.authority == nation_authority.key() @ SovereignError::InvalidAuthority
    )]
    pub nation: Account<'info, Nation>,
}

#[event]
pub struct UpdateNationRewardRateEvent {
    pub game_id: u64,
    pub nation_id: u8,
    pub gdp_reward_rate: u64,
    pub healthcare_reward_rate: u64,
    pub environment_reward_rate: u64,
    pub stability_reward_rate: u64,
    pub slot: u64,
}


pub fn mint_tokens_to_player_wallet(ctx: Context<MintTokensToPlayerWallet>, args: MintTokensToPlayerWalletArgs) -> Result<()> {
    require!(ctx.accounts.nation.is_alive, SovereignError::NationIsDead);

    ctx.accounts.player_wallet.balances[ctx.accounts.nation.nation_id as usize] = ctx.accounts.player_wallet.balances[ctx.accounts.nation.nation_id as usize].checked_add(args.amount).ok_or(SovereignError::MathOverflow)?;
    ctx.accounts.nation.minted_tokens_total = ctx.accounts.nation.minted_tokens_total.checked_add(args.amount).ok_or(SovereignError::MathOverflow)?;
    emit!(MintTokensToPlayerWalletEvent {
        game_id: ctx.accounts.nation.game_id,
        nation_id: ctx.accounts.nation.nation_id,
        player: args.player_authority.key().to_string(),
        player_wallet: ctx.accounts.player_wallet.key().to_string(),
        amount: args.amount,
        slot: Clock::get()?.slot,
    });

    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct MintTokensToPlayerWalletArgs {
    pub player_authority: Pubkey,
    pub amount: u64,
}

#[event]
pub struct MintTokensToPlayerWalletEvent {
    pub game_id: u64,
    pub nation_id: u8,
    pub player: String,
    pub player_wallet: String,
    pub amount: u64,
    pub slot: u64,
}

#[derive(Accounts)]
pub struct MintTokensToPlayerWallet<'info> {
    pub nation_authority: Signer<'info>,
    #[account(
        mut,    
        constraint = nation.authority == nation_authority.key() @ SovereignError::InvalidAuthority
    )]
    pub nation: Account<'info, Nation>,
    pub player_wallet: Account<'info, Wallet>,
}

pub fn deposit_to_broker(ctx: Context<DepositToBroker>, args: DepositToBrokerArgs) -> Result<()> {
    require!(ctx.accounts.nation.is_alive, SovereignError::NationIsDead);

    transfer(
        CpiContext::new(ctx.accounts.system_program.to_account_info(), Transfer {
            from: ctx.accounts.nation.to_account_info(),
            to: ctx.accounts.broker_escrow.to_account_info(),
        }),
        args.amount.checked_add(TXN_FEE).ok_or(SovereignError::MathOverflow)? // Pay for claim fee
    )?;

    emit!(DepositToBrokerEvent {
        game_id: ctx.accounts.game.id,
        nation_id: ctx.accounts.nation.nation_id,
        amount: args.amount,
    });

    Ok(())
}

#[event]
pub struct DepositToBrokerEvent {
    pub game_id: u64,
    pub nation_id: u8,
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositToBrokerArgs {
    pub amount: u64,
}

#[derive(Accounts)]
pub struct DepositToBroker<'info> {
    pub game_authority: Signer<'info>,
    pub nation_authority: Signer<'info>,
    #[account(
        mut,    
        constraint = nation.authority == nation_authority.key() @ SovereignError::InvalidAuthority
    )]
    pub nation: Account<'info, Nation>,
    #[account(
        constraint = game.authority == game_authority.key() @ SovereignError::InvalidAuthority,
        constraint = game.id == nation.game_id @ SovereignError::InvalidGameId
    )]
    pub game: Account<'info, Game>,
    #[account(
        mut,
        constraint = game.id == broker_escrow.game_id @ SovereignError::InvalidBroker
    )]
    pub broker_escrow: Account<'info, BrokerEscrow>,
    pub system_program: Program<'info, System>,
}

pub fn coup_nation(ctx: Context<CoupNation>) -> Result<()> {
    require!(ctx.accounts.nation.is_alive, SovereignError::NationIsDead);
    require!(ctx.accounts.game.nations_alive == 1, SovereignError::GameNotOver);

    ctx.accounts.nation.is_alive = false;
    ctx.accounts.game.nations_alive -= 1;

    // Transfer broker escrow $ and world agent $ to last nation
    let broker_escrow_signer_seeds = &[WALLET_SEED.as_bytes(), &ctx.accounts.game.id.to_le_bytes(), &ctx.accounts.game.broker_key.to_bytes(), &[ctx.bumps.broker_escrow]];
    transfer(
        CpiContext::new_with_signer (
            ctx.accounts.system_program.to_account_info(), 
            Transfer {
                from: ctx.accounts.broker_escrow.to_account_info(),
                to: ctx.accounts.nation.to_account_info(),
            },
            &[broker_escrow_signer_seeds]
        ),
        ctx.accounts.broker_escrow.to_account_info().lamports().checked_sub(TXN_FEE).ok_or(SovereignError::MathOverflow)?
    )?;

    let signer_seeds = &[WALLET_SEED.as_bytes(), &ctx.accounts.game.id.to_le_bytes(), &ctx.accounts.game.world_agent.to_bytes(), &[ctx.bumps.world_agent_wallet]];
    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(), 
            Transfer {
                from: ctx.accounts.world_agent_wallet.to_account_info(),
                to: ctx.accounts.nation.to_account_info(),
            },      
            &[signer_seeds]
        ),
        ctx.accounts.world_agent_wallet.to_account_info().lamports().checked_sub(TXN_FEE).ok_or(SovereignError::MathOverflow)?
    )?;

    emit!(CoupNationEvent{
        game_id: ctx.accounts.game.id,
    });

    Ok(())
}

#[event]
pub struct CoupNationEvent {
    pub game_id: u64,
}

#[derive(Accounts)]
pub struct CoupNation<'info> {
    pub game: Account<'info, Game>,
    #[account(
        mut,
        constraint = nation.game_id == game.id @ SovereignError::InvalidGameId
    )]
    pub nation: Account<'info, Nation>,
    /// CHECK: Checked in instruction
    #[account(address = nation.authority)]
    pub nation_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [BROKER_ESCROW_SEED.as_bytes(), &game.id.to_le_bytes()],
        bump
    )]
    pub broker_escrow: Account<'info, BrokerEscrow>,
    #[account(
        mut,
        seeds = [WALLET_SEED.as_bytes(), &game.id.to_le_bytes(), &game.world_agent.to_bytes()],
        bump
    )]
    pub world_agent_wallet: Box<Account<'info, Wallet>>,
    pub system_program: Program<'info, System>,
}

pub fn loot_nation(ctx: Context<LootNation>) -> Result<()> {
    require!(ctx.accounts.game.nations_alive == 0, SovereignError::GameNotOver);
    // Burn Nation Tokens from Wallet and Transfer Sol to Player Wallet
    let lamports_per_token = ctx.accounts.nation.to_account_info().lamports().checked_div(ctx.accounts.nation.minted_tokens_total).ok_or(SovereignError::MathOverflow)?;
    let lamports_to_transfer = ctx.accounts.player_wallet.balances[ctx.accounts.nation.nation_id as usize].checked_mul(lamports_per_token).ok_or(SovereignError::MathOverflow)?;

    let nation_signer_seeds = &[NATION_SEED.as_bytes(), &ctx.accounts.game.id.to_le_bytes(), &ctx.accounts.nation.authority.to_bytes(), &[ctx.bumps.nation]];
    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(), 
            Transfer {
                from: ctx.accounts.nation.to_account_info(),
                to: ctx.accounts.player_authority.to_account_info(),
            },
            &[nation_signer_seeds]
        ),
        lamports_to_transfer
    )?;

    ctx.accounts.player_wallet.balances[ctx.accounts.nation.nation_id as usize] = 0;

    emit!(LootNationEvent {
        game_id: ctx.accounts.game.id,
        nation_id: ctx.accounts.nation.nation_id,
        player_authority: ctx.accounts.player_authority.key().to_string(),
        amount: lamports_to_transfer,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct LootNation<'info> {
    #[account(
        seeds = [GAME_SEED.as_bytes(), &game.id.to_le_bytes()],
        bump
    )]
    pub game: Account<'info, Game>,
    #[account(
        mut,    
        constraint = nation.game_id == game.id @ SovereignError::InvalidGameId,
        seeds = [NATION_SEED.as_bytes(), &nation.game_id.to_le_bytes(), &nation.authority.to_bytes()],
        bump
    )]
    pub nation: Account<'info, Nation>,
    #[account(mut)]
    pub player_authority: Signer<'info>,
    #[account(
        mut,
        seeds = [WALLET_SEED.as_bytes(), &game.id.to_le_bytes(), player_authority.key().as_ref()],
        bump
    )]
    pub player_wallet: Account<'info, Wallet>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct LootNationEvent {
    pub game_id: u64,
    pub nation_id: u8,
    pub player_authority: String,
    pub amount: u64,
}