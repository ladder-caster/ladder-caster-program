use anchor_lang::prelude::*;

use instructions::*;

use crate::utils::validate_instruction_order_and_program_ids;
use crate::utils::validate_is_last_instructions_and_program_ids;
use crate::utils::{ActionType, ItemFeature, ItemType};

mod account;
mod config;
mod error;
mod event;
mod instructions;
mod utils;

//declare_id!("LCNTy2Q4HsKUecKEETwwzujjbAx9DGFJort8SjGzGFj");
// declare_id!("LCThBz55Ma7hcueUQA3iiofBhvidQHfNLxxwfLsycxb");
declare_id!("Aejjj86wGd3B4BEEaQyn4663xabWxLyNkJr6d7qLRcov");

#[program]
pub mod laddercast {
    use super::*;

    //********************************************
    //Initialization functions
    //********************************************
    pub fn init_game(ctx: Context<InitGame>, turn_info: GameTurnInfo) -> ProgramResult {
        init_game::init_game(ctx, turn_info)
    }

    pub fn init_player(ctx: Context<InitPlayer>) -> ProgramResult {
        init_player::init_player(ctx)
    }

    #[access_control(validate_is_last_instructions_and_program_ids(& ctx.accounts.instruction_sysvar_account, & ctx.program_id))]
    pub fn init_caster(ctx: Context<InitCaster>) -> ProgramResult {
        init_caster::init_caster(ctx)
    }

    //********************************************
    //Close functions
    //********************************************
    pub fn close_game(ctx: Context<CloseGame>) -> ProgramResult {
        close_game::close_game(ctx)
    }

    //********************************************
    //Turn based functions
    //********************************************

    pub fn caster_commit_loot(ctx: Context<Loot>) -> ProgramResult {
        caster_commit_loot::caster_commit_loot(ctx)
    }

    pub fn caster_commit_move(ctx: Context<Move>, lvl: u8, clm: u8) -> ProgramResult {
        caster_commit_move::caster_commit_move(ctx, lvl, clm)
    }

    pub fn caster_commit_craft(ctx: Context<Craft>) -> ProgramResult {
        caster_commit_craft::caster_commit_craft(ctx)
    }

    #[access_control(validate_is_last_instructions_and_program_ids(& ctx.accounts.instruction_sysvar_account, & ctx.program_id))]
    pub fn caster_commit_spell(ctx: Context<Spell>) -> ProgramResult {
        caster_commit_spell::caster_commit_spell(ctx)
    }

    #[access_control(validate_instruction_order_and_program_ids(& ctx.accounts.instruction_sysvar_account, & ctx.program_id, & ctx.accounts.caster.turn_commit, ActionType::Move))]
    pub fn caster_redeem_move<'info>(
        ctx: Context<'_, '_, '_, 'info, CasterRedeemMoveAction<'info>>,
    ) -> ProgramResult {
        caster_redeem_move::caster_redeem_move(ctx)
    }

    #[access_control(validate_instruction_order_and_program_ids(& ctx.accounts.instruction_sysvar_account, & ctx.program_id, & ctx.accounts.caster.turn_commit, ActionType::Loot))]
    pub fn caster_redeem_loot<'info>(
        ctx: Context<'_, '_, '_, 'info, CasterRedeemLootAction<'info>>,
    ) -> ProgramResult {
        caster_redeem_loot::caster_redeem_loot(ctx)
    }

    #[access_control(validate_instruction_order_and_program_ids(& ctx.accounts.instruction_sysvar_account, & ctx.program_id, & ctx.accounts.caster.turn_commit, ActionType::Craft))]
    pub fn caster_redeem_craft<'info>(
        ctx: Context<'_, '_, '_, 'info, CasterRedeemCraftAction<'info>>,
    ) -> ProgramResult {
        caster_redeem_craft::caster_redeem_craft(ctx)
    }

    #[access_control(validate_instruction_order_and_program_ids(& ctx.accounts.instruction_sysvar_account, & ctx.program_id, & ctx.accounts.caster.turn_commit, ActionType::Spell))]
    pub fn caster_redeem_spell<'info>(
        ctx: Context<'_, '_, '_, 'info, CasterRedeemSpellAction<'info>>,
    ) -> ProgramResult {
        caster_redeem_spell::caster_redeem_spell(ctx)
    }

    #[access_control(validate_instruction_order_and_program_ids(& ctx.accounts.instruction_sysvar_account, & ctx.program_id, & ctx.accounts.caster.turn_commit, ActionType::Reward))]
    pub fn caster_redeem_reward<'info>(
        ctx: Context<'_, '_, '_, 'info, CasterRedeemRewardAction<'info>>,
    ) -> ProgramResult {
        caster_redeem_rewards::caster_redeem_reward(ctx)
    }

    #[access_control(validate_is_last_instructions_and_program_ids(& ctx.accounts.instruction_sysvar_account, & ctx.program_id))]
    pub fn crank(ctx: Context<Crank>) -> ProgramResult {
        crank::crank(ctx)
    }

    //********************************************
    //Non-turn based functions
    //********************************************

    pub fn equip_item(ctx: Context<EquipUnequipItem>) -> ProgramResult {
        equipment::equip_item(ctx)
    }

    pub fn unequip_item(ctx: Context<EquipUnequipItem>) -> ProgramResult {
        equipment::unequip_item(ctx)
    }

    #[access_control(validate_is_last_instructions_and_program_ids(& ctx.accounts.instruction_sysvar_account, & ctx.program_id))]
    pub fn open_chest(ctx: Context<OpenChest>) -> ProgramResult {
        open_chest::open_chest(ctx)
    }

    pub fn manual_resource_burn(
        ctx: Context<ManualResourceBurn>,
        resource_type: ItemFeature,
        amount_to_burn: u64,
    ) -> ProgramResult {
        manual_resource_burn::manual_resource_burn(ctx, resource_type, amount_to_burn)
    }

    //********************************************
    //Functions to mint / burn into NFTs
    //********************************************
    pub fn mint_item(
        ctx: Context<MintItem>,
        item_type_str: String,
        item_level: u8,
        nft_uri: String,
        merkle_proof: Vec<[u8; 32]>,
    ) -> ProgramResult {
        mint_nft::mint_item(ctx, nft_uri, merkle_proof, item_type_str, item_level)
    }

    pub fn mint_caster(
        ctx: Context<MintCaster>,
        item_level: u8,
        nft_uri: String,
        merkle_proof: Vec<[u8; 32]>,
    ) -> ProgramResult {
        mint_nft::mint_caster(ctx, nft_uri, merkle_proof, item_level)
    }

    pub fn redeem_item(ctx: Context<RedeemItem>) -> ProgramResult {
        burn_nft::redeem_item(ctx)
    }

    pub fn redeem_caster(ctx: Context<RedeemCaster>) -> ProgramResult {
        burn_nft::redeem_caster(ctx)
    }

    pub fn update_merkle_root(
        ctx: Context<UpdateMerkleRoot>,
        item_type_str: String,
        item_level: u8,
        merkle_root_nft: [u8; 32],
    ) -> ProgramResult {
        update_merkle_root::update_merkle_root(ctx, merkle_root_nft, item_type_str, item_level)
    }

    //********************************************
    //Debug functions only for testing
    //********************************************
    pub fn give_resources(ctx: Context<GiveResources>, amount: u64) -> ProgramResult {
        test_helper::give_resources(ctx, amount)
    }

    pub fn give_lada(ctx: Context<GiveLada>, amount: u64) -> ProgramResult {
        test_helper::give_lada(ctx, amount)
    }

    pub fn give_item(ctx: Context<GiveItems>, item_type: ItemType, level: u8) -> ProgramResult {
        test_helper::give_item(ctx, item_type, level)
    }

    pub fn change_tile(
        ctx: Context<ChangeTile>,
        tile_type: TileType,
        lvl: u8,
        col: u8,
    ) -> ProgramResult {
        test_helper::change_tile(ctx, tile_type, lvl, col)
    }

    //********************************************
    //Clean testing data
    //********************************************
    pub fn burn_lada(ctx: Context<BurnLada>) -> ProgramResult {
        burn_lada::burn_lada(ctx)
    }

    // Fix problems
    pub fn fix_redeem_spell(ctx: Context<FixRedeemSpell>) -> ProgramResult {
        fix_redeem_spell::fix_redeem_spell(ctx)
    }
}
