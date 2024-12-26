use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    constant::{TXN_FEE, WALLET_SEED},
    error::SovereignError,
    state::{Game, Nation, Wallet},
};

pub fn world_disaster(ctx: Context<WorldDisaster>, args: WorldDisasterArgs) -> Result<()> {
    require!(
        ctx.accounts.game_account.nations_alive > 1,
        SovereignError::GameNotOver
    );

    let nation = &mut ctx.accounts.nation;
    // If nation is dead, transfer all it's SOL to the world agent
    if nation.gdp.checked_sub(args.gdp_damage).is_none() {
        // Nation is dead
        nation.is_alive = false;
        nation.gdp = 0;
    } else {
        nation.gdp -= args.gdp_damage;
    }

    if nation.healthcare.checked_sub(args.health_damage).is_none() {
        // Nation is dead
        nation.is_alive = false;
        nation.gdp = 0;
    } else {
        nation.healthcare -= args.health_damage;
    }

    if nation
        .environment
        .checked_sub(args.environment_damage)
        .is_none()
    {
        // Nation is dead
        nation.is_alive = false;
        nation.environment = 0;
    } else {
        nation.environment -= args.environment_damage;
    }

    if nation
        .stability
        .checked_sub(args.stability_damage)
        .is_none()
    {
        // Nation is dead
        nation.is_alive = false;
        nation.stability = 0;
    } else {
        nation.stability -= args.stability_damage;
    }

    if !nation.is_alive {
        ctx.accounts.game_account.nations_alive -= 1;
        emit!(NationDissolutionEvent {
            game_id: ctx.accounts.game_account.id,
            nation_id: nation.nation_id,
        });

        let nation_signer_seeds = &[
            WALLET_SEED.as_bytes(),
            &ctx.accounts.game_account.id.to_le_bytes(),
            &nation.authority.to_bytes(),
            &[ctx.bumps.nation],
        ];
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: nation.to_account_info(),
                    to: ctx.accounts.world_agent_wallet.to_account_info(),
                },
                &[nation_signer_seeds],
            ),
            nation
                .to_account_info()
                .lamports()
                .checked_sub(TXN_FEE)
                .ok_or(SovereignError::MathOverflow)?, // leave some for transfer fee
        )?;
    }

    emit!(WorldDisasterEvent {
        game_id: ctx.accounts.game_account.id,
        nation_id: nation.nation_id,
        nation_gdp: nation.gdp,
        nation_healthcare: nation.healthcare,
        nation_environment: nation.environment,
        nation_stability: nation.stability,
    });

    Ok(())
}

#[event]
pub struct WorldDisasterEvent {
    pub game_id: u64,
    pub nation_id: u8,
    pub nation_gdp: u64,
    pub nation_healthcare: u64,
    pub nation_environment: u64,
    pub nation_stability: u64,
}

#[event]
pub struct NationDissolutionEvent {
    pub game_id: u64,
    pub nation_id: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WorldDisasterArgs {
    pub gdp_damage: u64,
    pub health_damage: u64,
    pub environment_damage: u64,
    pub stability_damage: u64,
}

#[derive(Accounts)]
pub struct WorldDisaster<'info> {
    pub world_authority: Signer<'info>,
    #[account(
        mut,
        constraint = world_agent_wallet.authority == world_authority.key() @ SovereignError::InvalidAuthority,
        seeds = [WALLET_SEED.as_bytes(), &game_account.id.to_le_bytes(), &game_account.world_agent.to_bytes()],
        bump
    )]
    pub world_agent_wallet: Box<Account<'info, Wallet>>,
    #[account(
        mut,
        constraint = game_account.world_agent == world_authority.key() @ SovereignError::InvalidAuthority
    )]
    pub game_account: Box<Account<'info, Game>>,
    #[account(
        mut,
        constraint = nation.game_id == game_account.id @ SovereignError::InvalidGameId,
        seeds = [WALLET_SEED.as_bytes(), &game_account.id.to_le_bytes(), &nation.authority.to_bytes()],
        bump
    )]
    pub nation: Box<Account<'info, Nation>>,
    pub system_program: Program<'info, System>,
}

pub fn nation_boost(ctx: Context<NationBoost>, args: NationBoostArgs) -> Result<()> {
    let world_agent_wallet_signer_seeds = &[
        WALLET_SEED.as_bytes(),
        &ctx.accounts.game_account.id.to_le_bytes(),
        &ctx.accounts.game_account.world_agent.to_bytes(),
        &[ctx.bumps.world_agent_wallet],
    ];
    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.world_agent_wallet.to_account_info(),
                to: ctx.accounts.nation.to_account_info(),
            },
            &[world_agent_wallet_signer_seeds],
        ),
        args.lamports_amount,
    )?;

    emit!(NationBoostEvent {
        game_id: ctx.accounts.game_account.id,
        nation_id: ctx.accounts.nation.nation_id,
        lamports_amount: args.lamports_amount,
    });

    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct NationBoostArgs {
    pub lamports_amount: u64,
}

#[event]
pub struct NationBoostEvent {
    pub game_id: u64,
    pub nation_id: u8,
    pub lamports_amount: u64,
}

#[derive(Accounts)]
pub struct NationBoost<'info> {
    pub world_authority: Signer<'info>,
    #[account(
        mut,
        seeds = [WALLET_SEED.as_bytes(), &game_account.id.to_le_bytes(), &game_account.world_agent.to_bytes()],
        bump
    )]
    pub world_agent_wallet: Box<Account<'info, Wallet>>,
    #[account(
        constraint = game_account.world_agent == world_authority.key() @ SovereignError::InvalidAuthority
    )]
    pub game_account: Box<Account<'info, Game>>,
    #[account(
        mut,
        constraint = nation.game_id == game_account.id @ SovereignError::InvalidGameId
    )]
    pub nation: Box<Account<'info, Nation>>,
    /// CHECK: Checked in instruction
    #[account(address = nation.authority)]
    pub nation_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
