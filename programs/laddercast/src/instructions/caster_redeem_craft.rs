use anchor_lang::{prelude::*, solana_program::sysvar};

use crate::account::*;
use crate::error::ErrorCode;
use crate::utils::{ACTION_CRAFT_INDEX, generate_new_equipment, ItemRarity, MAX_LEVEL_1_BASED, RandomGenerator, zombify_account};

#[derive(Accounts)]
pub struct CasterRedeemCraftAction<'info> {
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub game: Box<Account<'info, Game>>,
    #[account(mut, has_one = authority, has_one = game)]
    pub player: Box<Account<'info, Player>>,
    #[account(mut, constraint = caster.owner == player.key())]
    pub caster: Box<Account<'info, Caster>>,

    #[account(address = sysvar::slot_hashes::id())]
    pub slots: UncheckedAccount<'info>,

    #[account(address = sysvar::instructions::id())]
    pub instruction_sysvar_account: UncheckedAccount<'info>,

    #[account(init, space = Item::SIZE, payer = authority)]
    pub item: Box<Account<'info, Item>>, //This will represent the item that will be newly crafted
}

pub fn caster_redeem_craft<'info>(
    ctx: Context<'_, '_, '_, 'info, CasterRedeemCraftAction<'info>>,
) -> ProgramResult {
    let caster = &mut ctx.accounts.caster;
    let game = &ctx.accounts.game;
    let player = &ctx.accounts.player;

    // Flag used to burn item account if not populated
    let mut burn_item_account = true;

    match caster.turn_commit {
        None => {
            return Err(ErrorCode::EmptyTurnCommit.into());
        }
        Some(mut turn_commit) => {
            if game.turn_info.turn == turn_commit.turn {
                return Err(ErrorCode::SameTurnRedeem.into());
            }

            let index_next_action = turn_commit.actions.get_next_action_to_be_executed();

            if ACTION_CRAFT_INDEX != index_next_action {
                return Err(ErrorCode::ActionOrderError.into());
            }

            let slots_ref = ctx.accounts.slots.data.borrow();
            let slots = &**slots_ref;

            let mut rand = RandomGenerator::new(slots, caster.to_account_info().key());

            //Crafting
            let crafting_snapshot = turn_commit.actions.crafting.unwrap();
            let spell_snapshot = turn_commit.actions.spell.clone();

            //Item level or rarity has a 10% chance of going up
            let mut new_item_level = crafting_snapshot.min_level;
            let mut new_item_rarity = crafting_snapshot.min_rarity;

            //If you have a spell that increases the level, it defaults to common for rarity
            if spell_snapshot != None
                && spell_snapshot.unwrap().is_extra_level_bonus
                && crafting_snapshot.min_level < MAX_LEVEL_1_BASED
            {
                new_item_level += 1;
                new_item_rarity = ItemRarity::Common;
            } else if rand.random_within_range::<u8, 1>(0, 10) == 5 {
                if rand.random_within_range::<u8, 1>(1, 2) == 1
                    && crafting_snapshot.min_level < MAX_LEVEL_1_BASED
                {
                    new_item_level += 1;
                    new_item_rarity = ItemRarity::Common;
                } else {
                    match new_item_rarity {
                        ItemRarity::Common => {
                            new_item_rarity = ItemRarity::Rare;
                        }
                        ItemRarity::Rare => {
                            new_item_rarity = ItemRarity::Epic;
                        }
                        ItemRarity::Epic => {
                            if crafting_snapshot.max_rarity
                                == ItemRarity::Legendary
                            {
                                new_item_rarity = ItemRarity::Legendary;
                            }
                        }
                        ItemRarity::Legendary => {}
                    }
                }
            }

            let item = &mut ctx.accounts.item;

            generate_new_equipment(
                item,
                game,
                player,
                new_item_level,
                Some(new_item_rarity),
                &mut rand,
            )?;
            burn_item_account = false;

            //Set to max, since we filter to get the min to find next action
            caster.turn_commit.as_mut().unwrap().actions.action_order[index_next_action] = 0;
        }
    }

    // Burn item if not used
    if burn_item_account {
        let item = &mut ctx.accounts.item;
        zombify_account(
            item,
            ctx.accounts.authority.to_account_info(),
            ctx.program_id,
        )?;
    }

    Ok(())
}
