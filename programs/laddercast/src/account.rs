use core::mem::size_of;

use anchor_lang::prelude::*;

use crate::{GameTurnInfo, Tile, TileType};
use crate::utils::ItemType;
use crate::utils::Modifiers;
use crate::utils::TurnCommit;

/// Signer PDA with seeds ["game_signer", signer_bump]
#[account]
pub struct Game {
    pub authority: Pubkey,
    pub map: [[Option<Tile>; 3]; 30],
    pub turn_info: GameTurnInfo,
    pub last_turn_added: u32,
    pub signer_bump: u8,
    /// Authority is game signer
    pub resource_1_mint_account: Pubkey,
    /// Authority is game signer
    pub resource_2_mint_account: Pubkey,
    /// Authority is game signer
    pub resource_3_mint_account: Pubkey,

    pub lada_mint_account: Pubkey,
    pub lada_token_account: Pubkey,
}

impl Game {
    pub const SIZE: usize =
        8 + 32 + (90 + 1) * Tile::SIZE + 8 + 1 + 32 + 32 + 32 + 32 + 32 + 32 + 300;

    pub fn get_map_as_tile_features_only(&self) -> [[Option<TileType>; 3]; 30] {
        let mut result: [[Option<TileType>; 3]; 30] = [[None; 3]; 30];

        for (i0, v0) in self.map.iter().enumerate() {
            for (i1, v1) in v0.iter().enumerate() {
                result[i0][i1] = match v1 {
                    None => None,
                    Some(tile) => Some((*tile).tile_type)
                }
            }
        }

        result
    }
}

/// Data about a specific turn
/// PDA with seeds ["turn_data", game.key, turn as string]
#[account]
pub struct TurnData {
    pub bump: u8,
    pub resource_1_burned: u64,
    pub resource_2_burned: u64,
    pub resource_3_burned: u64,
    //Backup of the map for that specific turn
    pub map: [[Option<TileType>; 3]; 30],
}

impl TurnData {
    pub const SIZE: usize = 8 + 1 + 8 + 8 + 8 + 90 * (1 + size_of::<TileType>());
}

impl Default for TurnData {
    fn default() -> Self {
        Self {
            bump: 0,
            resource_1_burned: 0,
            resource_2_burned: 0,
            resource_3_burned: 0,
            map: [[None; 3]; 30],
        }
    }
}

/// PDA with seeds ["merkle_roots", game.key, item_type, item_level]
/// For combined, item type = "combined" and item level = 0
#[account]
pub struct MerkleRootNFT {
    pub bump: u8,

    /// Roots of the Merkle Tree for dynamic NFT minting
    pub merkle_root_nft: [u8; 32],
}

impl MerkleRootNFT {
    pub const SIZE: usize = 8 + 1 + 32;
}

#[account]
pub struct MetadataNFTItem {
    pub self_bump: u8,
    pub mint_bump: u8,
    pub mint: Pubkey,
    pub item: MetadataItem,
}

impl MetadataNFTItem {
    pub const SIZE: usize = 8 + 1 + 1 + 32 + size_of::<MetadataItem>() + 300;
}

#[account]
pub struct MetadataNFTCaster {
    pub self_bump: u8,
    pub mint_bump: u8,
    pub mint: Pubkey,
    pub caster: MetadataCaster,
}

impl MetadataNFTCaster {
    pub const SIZE: usize = 8 + 1 + 1 + 32 + size_of::<MetadataCaster>() + 300;
}

#[account]
pub struct Player {
    pub authority: Pubkey,
    pub game: Pubkey,
    pub bump: u8,
}

impl Player {
    pub const SIZE: usize = 8 + 32 + 32 + 1 + 300;
}

#[account]
#[derive(Default, Copy)]
pub struct Caster {
    //Versioning will be done via seeds instead of via a property on the account itself
    //will make cleaner code that way. However, we generated the NFT metadata JSONs
    //using the version for the caster, so we have to keep it for that account only.
    pub version: u8,
    pub level: u8,
    pub experience: u64,
    /// Player
    pub owner: Pubkey,
    pub modifiers: Modifiers,
    /// If filled cannot unequip/equip
    pub turn_commit: Option<TurnCommit>,
}

impl Caster {
    pub const SIZE: usize =
        8 + 1 + 1 + 8 + 32 + size_of::<Modifiers>() + 1 + size_of::<TurnCommit>() + 300;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, Copy)]
pub struct MetadataItem {
    /// Game
    pub game: Pubkey,
    /// Player
    pub owner: Pubkey,
    pub level: u8,
    pub item_type: ItemType,
    /// Caster
    pub equipped_owner: Option<Pubkey>,
}

impl MetadataItem {
    pub const SIZE: usize = 8 + 32 + 32 + 1 + size_of::<ItemType>() + 33 + 300;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, Copy)]
pub struct MetadataCaster {
    pub version: u8,
    pub level: u8,
    pub experience: u64,
    /// Player
    pub owner: Pubkey,
    pub modifiers: Modifiers,
    /// If filled cannot unequip/equip
    pub turn_commit: Option<TurnCommit>,
}

impl MetadataCaster {
    pub const SIZE: usize =
        8 + 1 + 1 + 8 + 32 + size_of::<Modifiers>() + 1 + size_of::<TurnCommit>() + 300;
}

#[account]
#[derive(Default, Copy)]
pub struct Item {
    /// Game
    pub game: Pubkey,
    /// Player
    pub owner: Pubkey,
    pub level: u8,
    pub item_type: ItemType,
    /// Caster
    pub equipped_owner: Option<Pubkey>,
}

impl Item {
    pub const SIZE: usize = 8 + 32 + 32 + 1 + size_of::<ItemType>() + 33 + 300;
}
