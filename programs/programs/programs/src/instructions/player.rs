use std::fmt::Debug;

use anchor_lang::solana_program::hash::hash;
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use groth16_solana::groth16::Groth16Verifier;
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{CreateV2CpiBuilder, UpdatePluginV1CpiBuilder, BurnV1CpiBuilder},
    types::{
        Attribute, Attributes, FreezeDelegate, BurnDelegate, Plugin, PluginAuthority, PluginAuthorityPair,
        PluginType,
    },
    ID as MPL_CORE_ID,
};

use crate::{
    constant::{GAME_SEED, NATION_SEED, NATION_STATES, PLAYER_SEED, WALLET_SEED, STAKED_CITIZEN_SEED, BALANCES_INIT},
    error::SovereignError,
    state::{Bounty, BrokerEscrow, Game, Nation, Player, Wallet, StakedCitizen},
    verifying_key::VERIFYINGKEY,
};
type G1 = ark_bn254::g1::G1Affine;

pub fn register_player(ctx: Context<RegisterPlayer>, args: RegisterPlayerArgs) -> Result<()> {
    ctx.accounts.player.game_id = ctx.accounts.game.id;
    ctx.accounts.player.authority = ctx.accounts.player_authority.key();
    ctx.accounts.player.x_username = args.x_username;

    ctx.accounts.player_wallet.game_id = ctx.accounts.game.id;
    ctx.accounts.player_wallet.authority = ctx.accounts.player_authority.key();
    ctx.accounts.player_wallet.balances = BALANCES_INIT;

    emit!(RegisterPlayerEvent {
        game_id: ctx.accounts.game.id,
        authority: ctx.accounts.player_authority.key().to_string(),
        x_username: ctx.accounts.player.x_username.clone(),
    });

    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RegisterPlayerArgs {
    pub x_username: String,
}

#[derive(Accounts)]
pub struct RegisterPlayer<'info> {
    #[account(mut)]
    pub player_authority: Signer<'info>,
    pub game: Account<'info, Game>,
    #[account(
        init,
        payer = player_authority,
        space = 8 + Player::INIT_SPACE,
        seeds = [PLAYER_SEED.as_bytes(), &game.id.to_le_bytes(), &player_authority.key().as_ref()],
        bump
    )]
    pub player: Account<'info, Player>,
    #[account(
        init,
        payer = player_authority,
        space = 8 + Wallet::INIT_SPACE,
        seeds = [WALLET_SEED.as_bytes(), &game.id.to_le_bytes(), player_authority.key().as_ref()],
        bump
    )]
    pub player_wallet: Box<Account<'info, Wallet>>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct RegisterPlayerEvent {
    pub game_id: u64,
    pub authority: String,
    pub x_username: String,
}

pub fn mint_citizen(ctx: Context<MintCitizen>) -> Result<()> {
    // Mint Price is sent 100% to World Agent
    // World Agent will then send to Nation State Account (not authority) based on Citizen's Nation State
    // Cannot do it in this ix, because nation state is random, and we'd need its account.
    // We log it so it can be picked up by indexer and added to queue for World Agent to process

    transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.player_authority.to_account_info(),
                to: ctx.accounts.world_agent_wallet.to_account_info(),
            },
        ),
        ctx.accounts.game.mint_cost,
    )?;
    // Mint Price is sent 100% to World Agent as SOL
    ctx.accounts.world_agent_wallet.balances[0] = ctx.accounts.world_agent_wallet.balances[0].checked_add(ctx.accounts.game.mint_cost).ok_or(SovereignError::MathOverflow)?;

    let signers_seeds = &[
        GAME_SEED.as_bytes(),
        &ctx.accounts.game.id.to_le_bytes(),
        &[ctx.bumps.game],
    ];

    let clock = Clock::get()?;
    let mut hash_input = Vec::with_capacity(40); // 32 + 8 bytes
    hash_input.extend_from_slice(&ctx.accounts.player_authority.key().to_bytes());
    hash_input.extend_from_slice(&clock.slot.to_le_bytes());
    let hash = hash(&hash_input);
    let hash_bytes = hash.to_bytes(); // Convert Hash to [u8; 32]
    let gdp_fix = hash_bytes[0] as u8;
    let healthcare_fix = hash_bytes[1] as u8;
    let environment_fix = hash_bytes[2] as u8;
    let stability_fix = hash_bytes[3] as u8;
    let nation_state_idx = {
        let tmp = u32::from_be_bytes(clock.slot.to_be_bytes()[4..8].try_into().unwrap()) as usize % NATION_STATES.len();
        if tmp == 0 { 1 } else { tmp }
    };
    let nation_state = NATION_STATES[nation_state_idx];

    CreateV2CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.citizen_asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .authority(Some(&ctx.accounts.game.to_account_info()))
        .payer(&ctx.accounts.player_authority.to_account_info())
        .system_program(&ctx.accounts.system_program.to_account_info())
        .name(format!("Citizen - Game {:#}", ctx.accounts.game.id).to_string())
        .uri(ctx.accounts.collection.uri.to_string())
        .plugins(vec![
            PluginAuthorityPair {
                plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                authority: Some(PluginAuthority::UpdateAuthority),
            },
            PluginAuthorityPair {
                plugin: Plugin::Attributes(Attributes {
                    attribute_list: vec![
                        Attribute {
                            key: "game".to_string(),
                            value: ctx.accounts.game.id.to_string(),
                        },
                        Attribute {
                            key: "nation_state".to_string(),
                            value: nation_state.to_string(),
                        },
                        Attribute {
                            key: "gdp_fix".to_string(),
                            value: gdp_fix.to_string(),
                        },
                        Attribute {
                            key: "healthcare_fix".to_string(),
                            value: healthcare_fix.to_string(),
                        },
                        Attribute {
                            key: "environment_fix".to_string(),
                            value: environment_fix.to_string(),
                        },
                        Attribute {
                            key: "stability_fix".to_string(),
                            value: stability_fix.to_string(),
                        },
                    ],
                }),
                authority: Some(PluginAuthority::UpdateAuthority),
            },
            PluginAuthorityPair {
                plugin: Plugin::BurnDelegate(BurnDelegate {}),
                authority: Some(PluginAuthority::UpdateAuthority),
            }
        ])
        .invoke_signed(&[signers_seeds])?;

    emit!(MintCitizenEvent {
        game_id: ctx.accounts.game.id,
        player_authority: ctx.accounts.player_authority.key().to_string(),
        citizen_asset_id: ctx.accounts.citizen_asset.key().to_string(),
        nation_id: nation_state_idx as u8,
        mint_cost: ctx.accounts.game.mint_cost,
    });

    Ok(())
}

#[event]
pub struct MintCitizenEvent {
    pub game_id: u64,
    pub player_authority: String,
    pub citizen_asset_id: String,
    pub nation_id: u8,
    pub mint_cost: u64,
}

#[derive(Accounts)]
pub struct MintCitizen<'info> {
    #[account(mut)]
    pub player_authority: Signer<'info>,
    /// CHECK:
    #[account(
        mut,
        seeds = [WALLET_SEED.as_bytes(), &game.id.to_le_bytes(), &game.world_agent.to_bytes()],
        bump
    )]
    pub world_agent_wallet: Account<'info, Wallet>,
    #[account(
        mut,
        constraint = collection.key() == game.collection @ SovereignError::InvalidCollectionKey,
    )]
    pub collection: Account<'info, BaseCollectionV1>,
    #[account(mut)]
    pub citizen_asset: Signer<'info>,
    #[account(
        seeds = [GAME_SEED.as_bytes(), &game.id.to_le_bytes()],
        bump,
    )]
    pub game: Account<'info, Game>,
    /// CHECK: constraint checks it
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn stake_citizen(ctx: Context<StakeCitizen>, _args: StakeCitizenArgs) -> Result<()> {
    // Freeze the citizen
    let (_, freeze_plugin, _) = fetch_plugin::<BaseAssetV1, FreezeDelegate>(
        &ctx.accounts.citizen_asset.to_account_info(),
        PluginType::FreezeDelegate,
    )?;
    require!(!freeze_plugin.frozen, SovereignError::CitizenAlreadyStaked);
    let signers_seeds = &[
        GAME_SEED.as_bytes(),
        &ctx.accounts.game.id.to_le_bytes(),
        &[ctx.bumps.game],
    ];
    UpdatePluginV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.citizen_asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: true }))
        .authority(Some(&ctx.accounts.game.to_account_info()))
        .payer(&ctx.accounts.player_authority.to_account_info())
        .system_program(&ctx.accounts.system_program.to_account_info())
        .invoke_signed(&[signers_seeds])?;

    // Create StakedCitizen account
    let staked_citizen = &mut ctx.accounts.staked_citizen;
    staked_citizen.owner = ctx.accounts.player_authority.key();
    staked_citizen.citizen_asset_id = ctx.accounts.citizen_asset.key();
    staked_citizen.game_id = ctx.accounts.game.id;
    staked_citizen.nation_id = ctx.accounts.nation.nation_id;
    let (_, attribute_plugin, _) = fetch_plugin::<BaseAssetV1, Attributes>(
        &ctx.accounts.citizen_asset.to_account_info(),
        PluginType::Attributes,
    )?;
    let mut rewards = (0, 0, 0, 0); // (gdp, healthcare, environment, stability)
    let mut found = (false, false, false, false); // Make sure they are all found

    for attr in attribute_plugin.attribute_list.iter() {
        match attr.key.as_str() {
            "gdp_fix" => {
                found.0 = true;
                rewards.0 = attr.value
                    .parse::<u64>()
                    .map_err(|_| SovereignError::InvalidCitizenAttributeValue)?
                    .checked_mul(ctx.accounts.nation.gdp_reward_rate)
                    .ok_or(SovereignError::MathOverflow)?;
            },
            "healthcare_fix" => {
                found.1 = true;
                rewards.1 = attr.value
                    .parse::<u64>()
                    .map_err(|_| SovereignError::InvalidCitizenAttributeValue)?
                    .checked_mul(ctx.accounts.nation.healthcare_reward_rate)
                    .ok_or(SovereignError::MathOverflow)?;
            },
            "environment_fix" => {
                found.2 = true;
                rewards.2 = attr.value
                    .parse::<u64>()
                    .map_err(|_| SovereignError::InvalidCitizenAttributeValue)?
                    .checked_mul(ctx.accounts.nation.environment_reward_rate)
                    .ok_or(SovereignError::MathOverflow)?;
            },
            "stability_fix" => {
                found.3 = true;
                rewards.3 = attr.value
                    .parse::<u64>()
                    .map_err(|_| SovereignError::InvalidCitizenAttributeValue)?
                    .checked_mul(ctx.accounts.nation.stability_reward_rate)
                    .ok_or(SovereignError::MathOverflow)?;
            },
            _ => continue,
        }
    }

    // Check if we found all required attributes
    if !found.0 || !found.1 || !found.2 || !found.3 {
        return Err(SovereignError::CitizenAttributeNotFound.into());
    }

    staked_citizen.reward_amount = rewards.0
    .checked_add(rewards.1)
    .and_then(|sum| sum.checked_add(rewards.2))
    .and_then(|sum| sum.checked_add(rewards.3))
    .ok_or(SovereignError::MathOverflow)?;

    let clock = Clock::get()?;
    let complete_slot = clock.slot + ctx.accounts.game.citizen_stake_length;
    staked_citizen.complete_slot = complete_slot;

    emit!(StakeCitizenEvent {
        game_id: ctx.accounts.game.id,
        player_authority: ctx.accounts.player_authority.key().to_string(),
        citizen_asset_id: ctx.accounts.citizen_asset.key().to_string(),
        nation_id: ctx.accounts.nation.nation_id,
        complete_slot: complete_slot,
    });

    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct StakeCitizenArgs {}

#[derive(Accounts)]
pub struct StakeCitizen<'info> {
    #[account(mut)]
    pub player_authority: Signer<'info>,
    #[account(
        mut,
        constraint = citizen_asset.owner == player_authority.key() @ SovereignError::InvalidCitizenAsset
    )]
    pub citizen_asset: Account<'info, BaseAssetV1>,
    #[account(
        seeds = [GAME_SEED.as_bytes(), &game.id.to_le_bytes()],
        bump,
    )]
    pub game: Account<'info, Game>,
    #[account(
        seeds = [
            NATION_SEED.as_bytes(), 
            &game.id.to_le_bytes(), 
            &nation.nation_id.to_le_bytes()
        ],
        bump,
        constraint = nation.is_alive @ SovereignError::NationIsDead
    )]
    pub nation: Account<'info, Nation>,
    #[account(
        init,
        payer = player_authority,
        space = 8 + StakedCitizen::INIT_SPACE,
        seeds = [
            STAKED_CITIZEN_SEED.as_bytes(),
            &game.id.to_le_bytes(),
            &nation.nation_id.to_le_bytes(),
            citizen_asset.key().as_ref()
        ],
        bump
    )]
    pub staked_citizen: Account<'info, StakedCitizen>,
    pub collection: Account<'info, BaseCollectionV1>,
    /// CHECK: constraint checks it
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct StakeCitizenEvent {
    pub game_id: u64,
    pub player_authority: String,
    pub citizen_asset_id: String,
    pub nation_id: u8,
    pub complete_slot: u64,
}
pub fn complete_stake(ctx: Context<CompleteStake>, _args: CompleteStakeArgs) -> Result<()> {
    // Check that citizen_asset is frozen
    let (_, freeze_plugin, _) = fetch_plugin::<BaseAssetV1, FreezeDelegate>(
        &ctx.accounts.citizen_asset.to_account_info(),
        PluginType::FreezeDelegate,
    )?;
    require!(freeze_plugin.frozen, SovereignError::CitizenNotStaked);

    let staked_citizen = &ctx.accounts.staked_citizen;
    let slot = Clock::get()?.slot;
    // If nation is alive, check if stake is complete (otherwise error) and apply benefits
    // If nation is dead, just continue to unfreeze citizen_asset
    if ctx.accounts.nation.is_alive {
        require!(
            slot >= staked_citizen.complete_slot,
            SovereignError::StakeNotComplete
        );

        // Player recieves rewards
        ctx.accounts.player_wallet.balances[ctx.accounts.nation.nation_id as usize] = ctx.accounts.player_wallet.balances[ctx.accounts.nation.nation_id as usize].checked_add(staked_citizen.reward_amount).ok_or(SovereignError::MathOverflow)?;
        ctx.accounts.nation.minted_tokens_total = ctx.accounts.nation.minted_tokens_total.checked_add(staked_citizen.reward_amount).ok_or(SovereignError::MathOverflow)?;   

        // Nation recieves stat buffs
        // Collect all the fix values
        let (_, attribute_plugin, _) = fetch_plugin::<BaseAssetV1, Attributes>(
            &ctx.accounts.citizen_asset.to_account_info(),
            PluginType::Attributes,
        )?;
        let mut fix_values = (0, 0, 0, 0); // (gdp, healthcare, environment, stability)
        let mut found = (false, false, false, false); // Make sure they are all found
        for attr in attribute_plugin.attribute_list.iter() {
            match attr.key.as_str() {
                "gdp_fix" => {
                    found.0 = true;
                    fix_values.0 = attr.value
                        .parse::<u64>()
                        .map_err(|_| SovereignError::InvalidCitizenAttributeValue)?;
                },
                "healthcare_fix" => {
                    found.1 = true;
                    fix_values.1 = attr.value
                        .parse::<u64>()
                        .map_err(|_| SovereignError::InvalidCitizenAttributeValue)?;
                },
                "environment_fix" => {
                    found.2 = true;
                    fix_values.2 = attr.value
                        .parse::<u64>()
                        .map_err(|_| SovereignError::InvalidCitizenAttributeValue)?;
                },
                "stability_fix" => {
                    found.3 = true;
                    fix_values.3 = attr.value
                        .parse::<u64>()
                        .map_err(|_| SovereignError::InvalidCitizenAttributeValue)?;
                },
                _ => continue,
            }
        }

        // Check if we found all required attributes
        if !found.0 || !found.1 || !found.2 || !found.3 {
            return Err(SovereignError::CitizenAttributeNotFound.into());
        }

        ctx.accounts.nation.gdp = ctx.accounts.nation.gdp
            .checked_add(fix_values.0)
            .ok_or(SovereignError::MathOverflow)?;
    
        ctx.accounts.nation.healthcare = ctx.accounts.nation.healthcare
            .checked_add(fix_values.1)
            .ok_or(SovereignError::MathOverflow)?;
        
        ctx.accounts.nation.environment = ctx.accounts.nation.environment
            .checked_add(fix_values.2)
            .ok_or(SovereignError::MathOverflow)?;
        
        ctx.accounts.nation.stability = ctx.accounts.nation.stability
            .checked_add(fix_values.3)
            .ok_or(SovereignError::MathOverflow)?;
    }

    // Unfreeze citizen_asset
    let signers_seeds = &[
        GAME_SEED.as_bytes(),
        &ctx.accounts.game.id.to_le_bytes(),
        &[ctx.bumps.game],
    ];
    UpdatePluginV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.citizen_asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
        .authority(Some(&ctx.accounts.game.to_account_info()))
        .payer(&ctx.accounts.player_authority.to_account_info())
        .system_program(&ctx.accounts.system_program.to_account_info())
        .invoke_signed(&[signers_seeds])?;

    emit!(CompleteStakeEvent {
        game_id: ctx.accounts.game.id,
        player_authority: ctx.accounts.player_authority.key().to_string(),
        citizen_asset_id: ctx.accounts.citizen_asset.key().to_string(),
        nation_id: ctx.accounts.nation.nation_id,
        reward_amount: staked_citizen.reward_amount,
        slot: slot,
        nation_gdp: ctx.accounts.nation.gdp,
        nation_healthcare: ctx.accounts.nation.healthcare,
        nation_environment: ctx.accounts.nation.environment,
        nation_stability: ctx.accounts.nation.stability,
    });

    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CompleteStakeArgs{
}

#[derive(Accounts)]
pub struct CompleteStake<'info> {
    #[account(mut)]
    pub player_authority: Signer<'info>,
    #[account(
        mut,
        constraint = citizen_asset.owner == player_authority.key() @ SovereignError::InvalidCitizenAsset
    )]
    pub citizen_asset: Account<'info, BaseAssetV1>,
    #[account(
        seeds = [GAME_SEED.as_bytes(), &game.id.to_le_bytes()],
        bump,
    )]
    pub game: Account<'info, Game>,
    #[account(
        seeds = [
            NATION_SEED.as_bytes(), 
            &game.id.to_le_bytes(), 
            &nation.nation_id.to_le_bytes()
        ],
        bump,
        constraint = nation.is_alive @ SovereignError::NationIsDead
    )]
    pub nation: Account<'info, Nation>,
    #[account(
        mut,
        seeds = [
            STAKED_CITIZEN_SEED.as_bytes(),
            &game.id.to_le_bytes(), 
            &nation.nation_id.to_le_bytes(),
            citizen_asset.key().as_ref()
        ],
        bump,
        constraint = staked_citizen.citizen_asset_id == citizen_asset.key() @ SovereignError::InvalidCitizenAsset,
        close = player_authority
    )]
    pub staked_citizen: Account<'info, StakedCitizen>,
    #[account(mut)]
    pub player_wallet: Box<Account<'info, Wallet>>,
    pub collection: Account<'info, BaseCollectionV1>,
    /// CHECK: constraint checks it
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct CompleteStakeEvent {
    pub game_id: u64,
    pub player_authority: String,
    pub citizen_asset_id: String,
    pub nation_id: u8,
    pub reward_amount: u64,
    pub slot: u64,
    pub nation_gdp: u64,
    pub nation_healthcare: u64,
    pub nation_environment: u64,
    pub nation_stability: u64,
}

pub fn claim_bounty(ctx: Context<ClaimBounty>, args: ClaimBountyArgs) -> Result<()> {
    // Check that bounty has not expired
    require!(
        ctx.accounts.bounty.expiry_slot > Clock::get()?.slot,
        SovereignError::BountyExpired
    );

    // Check that citizen_asset is frozen
    let (_, freeze_plugin, _) = fetch_plugin::<BaseAssetV1, FreezeDelegate>(
        &ctx.accounts.citizen_asset.to_account_info(),
        PluginType::FreezeDelegate,
    )?;
    require!(freeze_plugin.frozen, SovereignError::CitizenNotStaked);

    verify_bounty(
        ctx.accounts.bounty.bounty_hash,
        args.bounty_nonce,
        ctx.accounts.game.bounty_pow_threshold,
        args.bounty_proof,
    )?;

    // Burn citizen asset
    let signers_seeds = &[
        GAME_SEED.as_bytes(),
        &ctx.accounts.game.id.to_le_bytes(),
        &[ctx.bumps.game],
    ];
    BurnV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.citizen_asset.to_account_info())
        .authority(Some(&ctx.accounts.game.to_account_info()))
        .payer(&ctx.accounts.player_authority.to_account_info())
        .system_program(Some(&ctx.accounts.system_program.to_account_info()))
        .invoke_signed(&[signers_seeds])?;

    // Transfer bounty to player
    let broker_escrow_signer_seeds = &[
        WALLET_SEED.as_bytes(),
        &ctx.accounts.game.id.to_le_bytes(),
        &ctx.accounts.game.broker_key.to_bytes(),
        &[ctx.bumps.broker_escrow],
    ];
    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.broker_escrow.to_account_info(),
                to: ctx.accounts.player_authority.to_account_info(),
            },
            &[broker_escrow_signer_seeds],
        ),
        ctx.accounts.bounty.amount,
    )?;

    let bounty_hash_str = ctx
        .accounts
        .bounty
        .bounty_hash
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    emit!(ClaimBountyEvent {
        game_id: ctx.accounts.game.id,
        bounty_hash: bounty_hash_str,
        player_authority: ctx.accounts.player_authority.key().to_string(),
        amount: ctx.accounts.bounty.amount,
        citizen_asset_id: ctx.accounts.citizen_asset.key().to_string(),
    });

    Ok(())
}

#[event]
pub struct ClaimBountyEvent {
    pub game_id: u64,
    pub bounty_hash: String,
    pub player_authority: String,
    pub amount: u64,
    pub citizen_asset_id: String,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ClaimBountyArgs {
    pub bounty_nonce: [u8; 32],
    pub bounty_proof: [u8; 256],
}

#[derive(Accounts)]
#[instruction(args: ClaimBountyArgs)]
pub struct ClaimBounty<'info> {
    #[account(mut)]
    pub player_authority: Signer<'info>,
    #[account(
        mut,
        seeds = [WALLET_SEED.as_bytes(), &game.id.to_le_bytes(), &game.broker_key.to_bytes()],
        bump
    )]
    pub broker_escrow: Account<'info, BrokerEscrow>,
    #[account(
        mut,
        constraint = bounty.game_id == game.id @ SovereignError::InvalidGameId,
        close = broker_escrow
    )]
    pub bounty: Account<'info, Bounty>,
    #[account(
        seeds = [
            NATION_SEED.as_bytes(), 
            &game.id.to_le_bytes(), 
            &nation.nation_id.to_le_bytes()
        ],
        bump,
    )]
    pub nation: Account<'info, Nation>,
    pub citizen_asset: Account<'info, BaseAssetV1>,
    #[account(
        mut,
        seeds = [
            STAKED_CITIZEN_SEED.as_bytes(),
            &game.id.to_le_bytes(),
            &nation.nation_id.to_le_bytes(),
            citizen_asset.key().as_ref()
        ],
        bump,
        constraint = staked_citizen.citizen_asset_id == citizen_asset.key() @ SovereignError::InvalidCitizenAsset,
        close = player_authority
    )]
    pub staked_citizen: Account<'info, StakedCitizen>,
    #[account(
        seeds = [GAME_SEED.as_bytes(), &game.id.to_le_bytes()],
        bump,
    )]
    pub game: Account<'info, Game>,
    /// CHECK: constraint checks it
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

fn verify_bounty(
    bounty_hash: [u8; 32],
    bounty_nonce: [u8; 32],
    threshold: [u8; 32],
    proof: [u8; 256],
) -> Result<()> {
    let proof_a: G1 = <G1 as CanonicalDeserialize>::deserialize_uncompressed(
        &*[&change_endianness(&proof[0..64])[..], &[0u8][..]].concat(),
    )
    .unwrap();
    let mut proof_a_neg = [0u8; 65];
    <G1 as CanonicalSerialize>::serialize_uncompressed(&-proof_a, &mut proof_a_neg[..]).unwrap();
    let proof_a: [u8; 64] = change_endianness(&proof_a_neg[..64]).try_into().unwrap();
    let proof_b: [u8; 128] = proof[64..192].try_into().unwrap();
    let proof_c: [u8; 64] = proof[192..256].try_into().unwrap();

    let public_inputs = [bounty_hash, bounty_nonce, threshold];
    let mut verifier =
        Groth16Verifier::new(&proof_a, &proof_b, &proof_c, &public_inputs, &VERIFYINGKEY).unwrap();

    verifier.verify().unwrap();
    Ok(())
}

fn change_endianness(bytes: &[u8]) -> Vec<u8> {
    let mut vec = Vec::new();
    for b in bytes.chunks(32) {
        for byte in b.iter().rev() {
            vec.push(*byte);
        }
    }
    vec
}
