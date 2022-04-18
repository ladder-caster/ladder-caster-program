use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::account::*;
use crate::error::ErrorCode;
use crate::TileType;
use crate::utils::{ACTION_LOOT_INDEX, DEFAULT_MAGIC_FIND_IN_PERCENT, get_current_tile_feature, get_player_bonuses, ItemType, RandomGenerator, zombify_account};

#[derive(Accounts)]
pub struct CasterRedeemLootAction<'info> {
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub game: Box<Account<'info, Game>>,
    #[account(mut, has_one = authority, has_one = game)]
    pub player: Box<Account<'info, Player>>,
    #[account(mut, constraint = caster.owner == player.key())]
    pub caster: Box<Account<'info, Caster>>,

    #[account(mut, seeds = [b"game_signer"], bump)]
    pub game_signer: UncheckedAccount<'info>,

    #[account(address = sysvar::slot_hashes::id())]
    pub slots: UncheckedAccount<'info>,

    #[account(address = sysvar::instructions::id())]
    pub instruction_sysvar_account: UncheckedAccount<'info>,

    #[account(mut, constraint = resource_1_mint_account.to_account_info().key() == game.resource_1_mint_account)]
    pub resource_1_mint_account: Box<Account<'info, Mint>>,
    #[account(mut, constraint = resource_2_mint_account.to_account_info().key() == game.resource_2_mint_account)]
    pub resource_2_mint_account: Box<Account<'info, Mint>>,
    #[account(mut, constraint = resource_3_mint_account.to_account_info().key() == game.resource_3_mint_account)]
    pub resource_3_mint_account: Box<Account<'info, Mint>>,

    #[account(init_if_needed,
    associated_token::mint = resource_1_mint_account,
    associated_token::authority = authority,
    payer = authority)]
    pub resource_1_token_account: Box<Account<'info, TokenAccount>>,
    #[account(init_if_needed,
    associated_token::mint = resource_2_mint_account,
    associated_token::authority = authority,
    payer = authority)]
    pub resource_2_token_account: Box<Account<'info, TokenAccount>>,
    #[account(init_if_needed,
    associated_token::mint = resource_3_mint_account,
    associated_token::authority = authority,
    payer = authority)]
    pub resource_3_token_account: Box<Account<'info, TokenAccount>>,

    //Got to do -1 to the turn because we want the turn BEFORE the crank
    #[account(mut, seeds = [
    b"turn_data",
    game.to_account_info().key().as_ref(),
    (caster.turn_commit.unwrap().turn).to_string().as_ref()
    ], bump = game_turn_data.bump)]
    pub game_turn_data: Box<Account<'info, TurnData>>,

    #[account(init, space = Item::SIZE, payer = authority)]
    pub item: Box<Account<'info, Item>>, //This will represent the chest that could potentially be found

    // Optional accounts for player bonuses
    pub staff: UncheckedAccount<'info>,
    pub head: UncheckedAccount<'info>,
    pub robe: UncheckedAccount<'info>,
}

pub fn caster_redeem_loot<'info>(
    ctx: Context<'_, '_, '_, 'info, CasterRedeemLootAction<'info>>,
) -> ProgramResult {
    let caster = &mut ctx.accounts.caster;
    let game = &ctx.accounts.game;
    let player = &ctx.accounts.player;
    let turn_data = &ctx.accounts.game_turn_data;

    // Flag used to burn item account if not populated
    let mut burn_item_account = true;

    match caster.turn_commit {
        None => {
            return Err(ErrorCode::EmptyTurnCommit.into());
        }
        Some(turn_commit) => {
            if game.turn_info.turn == turn_commit.turn {
                return Err(ErrorCode::SameTurnRedeem.into());
            }

            let index_next_action = turn_commit.actions.get_next_action_to_be_executed();

            if ACTION_LOOT_INDEX != index_next_action {
                return Err(ErrorCode::ActionOrderError.into());
            }

            let slots_ref = ctx.accounts.slots.data.borrow();
            let slots = &**slots_ref;

            let mut rand = RandomGenerator::new(slots, caster.to_account_info().key());

            let seeds = &[b"game_signer".as_ref(), &[ctx.accounts.game.signer_bump]];

            let signer = &[&seeds[..]];

            let tile_level = caster.modifiers.tile_level;
            let potential_looted_tile_type: Option<&TileType> = get_current_tile_feature(
                &turn_data.map,
                tile_level,
                caster.modifiers.tile_column,
            );

            if potential_looted_tile_type == None {
                return Err(ErrorCode::TileNotExists.into());
            }

            let looted_tile_type = potential_looted_tile_type.unwrap();

            let range_min_resource: u64 = 1;
            let mut range_max_resource: u64 = 10 * (tile_level + 1) as u64; // +1 since 0 based

            let staff_account: Result<Account<Item>, ProgramError> =
                Account::try_from(&ctx.accounts.staff);

            let head_account: Result<Account<Item>, ProgramError> =
                Account::try_from(&ctx.accounts.head);

            let robe_account: Result<Account<Item>, ProgramError> =
                Account::try_from(&ctx.accounts.robe);

            let player_bonuses = get_player_bonuses(
                &caster.modifiers,
                vec![&staff_account, &head_account, &robe_account],
                &game,
                &player,
                &caster,
            );

            match looted_tile_type {
                TileType::Earth => {
                    range_max_resource += player_bonuses.earth_chance as u64;
                }
                TileType::Fire => {
                    range_max_resource += player_bonuses.fire_chance as u64;
                }
                TileType::Water => {
                    range_max_resource += player_bonuses.water_chance as u64;
                }
                _ => {}
            }

            let mut number_of_resources_given = rand.random_within_range::<u64, 8>(range_min_resource, range_max_resource);

            if rand.random_within_range::<u16, 2>(100, 10_000)
                < player_bonuses.critical_chance
            {
                number_of_resources_given *= 2;
            }

            let resource_token_account: &Account<TokenAccount>;
            let resource_mint_account: &Account<Mint>;

            match looted_tile_type {
                TileType::Fire => {
                    resource_token_account =
                        &ctx.accounts.resource_1_token_account;
                    resource_mint_account =
                        &ctx.accounts.resource_1_mint_account;
                }
                TileType::Water => {
                    resource_token_account =
                        &ctx.accounts.resource_2_token_account;
                    resource_mint_account =
                        &ctx.accounts.resource_2_mint_account;
                }
                TileType::Earth => {
                    resource_token_account =
                        &ctx.accounts.resource_3_token_account;
                    resource_mint_account =
                        &ctx.accounts.resource_3_mint_account;
                }
                _ => {
                    return Err(ErrorCode::InvalidTileForLooting.into());
                }
            }

            token::mint_to(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info().clone(),
                    token::MintTo {
                        mint: resource_mint_account.to_account_info(),
                        to: resource_token_account.to_account_info(),
                        authority: ctx.accounts.game_signer.to_account_info(),
                    },
                )
                    .with_signer(signer),
                number_of_resources_given,
            )?;

            //Chance of finding a chest is 10% on a resource tile
            match looted_tile_type {
                TileType::Fire | TileType::Water | TileType::Earth => {
                    //default is 10% so 1000 since we work in % (to not have floating)
                    let magic_find_chance =
                        DEFAULT_MAGIC_FIND_IN_PERCENT + player_bonuses.magic_find_chance;

                    if rand.random_within_range::<u16, 2>(100, 10_000)
                        < magic_find_chance
                    {
                        let item = &mut ctx.accounts.item;
                        item.game = game.key();
                        item.owner = ctx.accounts.player.key();
                        item.equipped_owner = None;
                        item.item_type = ItemType::Chest {
                            tier: match tile_level {
                                0..=4 => 1,
                                5..=9 => 2,
                                10..=14 => 3,
                                15..=30 => 4, //29 should be the max tile level since 0 based
                                _ => 1,
                            },
                        };
                        //Since 0 based, +1
                        item.level = tile_level + 1;
                        burn_item_account = false;
                    }
                }
                _ => {}
            }

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
