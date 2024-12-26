use anchor_lang::prelude::*;

// Seeds = [PLAYER_SEED.as_bytes(), &game_id.to_le_bytes(), &authority.to_bytes()]
#[account]
#[derive(InitSpace)]
pub struct Player {
    pub game_id: u64,
    pub authority: Pubkey,
    #[max_len(32)]
    pub x_username: String,
}

// Seeds = [STAKED_CITIZEN_SEED.as_bytes(), &game.id.to_le_bytes(), &nation.nation_id.to_le_bytes(), citizen_asset.key().as_ref()]
#[account]
#[derive(InitSpace)]
pub struct StakedCitizen {
    pub citizen_asset_id: Pubkey,
    pub owner: Pubkey,
    pub nation_id: u8,
    pub game_id: u64,
    pub reward_amount: u64,
    pub complete_slot: u64, // the slot when the stake is complete
}
