use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::account::*;
use crate::error::ErrorCode;
use crate::utils::{
    generate_new_equipment, give_exp_to_caster_spell, is_spell_successful, zombify_account,
    ItemType, RandomGenerator, SpellType, ACTION_SPELL_INDEX,
};

#[derive(Accounts)]
pub struct CasterRedeemSpellAction<'info> {
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

    #[account(init, space = Item::SIZE, payer = authority)]
    pub item: Box<Account<'info, Item>>, //This will represent the item that could potentially be created by the casted spell

                                         //There will be a remaining account that represents the spell book item, so that we can
                                         //zero it out (burn it) at index 0
                                         //We will also use sub instructions for this,
}

pub fn caster_redeem_spell<'info>(
    ctx: Context<'_, '_, '_, 'info, CasterRedeemSpellAction<'info>>,
) -> ProgramResult {
    let caster = &mut ctx.accounts.caster;
    let game = &ctx.accounts.game;

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

            let slots_ref = ctx.accounts.slots.data.borrow();
            let slots = &**slots_ref;

            let mut rand = RandomGenerator::new(slots, caster.to_account_info().key());

            let index_next_action = turn_commit.actions.get_next_action_to_be_executed();

            if ACTION_SPELL_INDEX != index_next_action {
                return Err(ErrorCode::ActionOrderError.into());
            }

            let seeds = &[b"game_signer".as_ref(), &[ctx.accounts.game.signer_bump]];

            let signer = &[&seeds[..]];

            if ctx.remaining_accounts.is_empty() {
                return Err(ErrorCode::SpellAccountMissing.into());
            }

            if ctx.remaining_accounts.get(0).is_none() {
                return Err(ErrorCode::ProvidedSpellBookIsNull.into());
            }

            let spell_book_account = &mut ctx.remaining_accounts.get(0).unwrap();

            if spell_book_account.key() != caster.modifiers.spell_book.unwrap() {
                return Err(ErrorCode::SpellKeyMismatch.into());
            }

            let mut spell_book_account: &mut Account<Item> =
                &mut Account::try_from(&spell_book_account.clone()).unwrap();

            if let ItemType::SpellBook {
                spell,
                value,
                rarity,
                ..
            } = spell_book_account.item_type
            {
                if is_spell_successful(&mut rand, rarity) {
                    match spell {
                        SpellType::Fire => {
                            token::mint_to(
                                CpiContext::new(
                                    ctx.accounts.token_program.to_account_info().clone(),
                                    token::MintTo {
                                        mint: ctx
                                            .accounts
                                            .resource_1_mint_account
                                            .to_account_info(),
                                        to: ctx.accounts.resource_1_token_account.to_account_info(),
                                        authority: ctx.accounts.game_signer.to_account_info(),
                                    },
                                )
                                .with_signer(signer),
                                value as u64,
                            )?;
                        }
                        SpellType::Water => {
                            token::mint_to(
                                CpiContext::new(
                                    ctx.accounts.token_program.to_account_info().clone(),
                                    token::MintTo {
                                        mint: ctx
                                            .accounts
                                            .resource_2_mint_account
                                            .to_account_info(),
                                        to: ctx.accounts.resource_2_token_account.to_account_info(),
                                        authority: ctx.accounts.game_signer.to_account_info(),
                                    },
                                )
                                .with_signer(signer),
                                value as u64,
                            )?;
                        }
                        SpellType::Earth => {
                            token::mint_to(
                                CpiContext::new(
                                    ctx.accounts.token_program.to_account_info().clone(),
                                    token::MintTo {
                                        mint: ctx
                                            .accounts
                                            .resource_3_mint_account
                                            .to_account_info(),
                                        to: ctx.accounts.resource_3_token_account.to_account_info(),
                                        authority: ctx.accounts.game_signer.to_account_info(),
                                    },
                                )
                                .with_signer(signer),
                                value as u64,
                            )?;
                        }
                        SpellType::Experience => {
                            give_exp_to_caster_spell(caster, value as u64);
                        }
                        SpellType::Item => {
                            let item = &mut ctx.accounts.item;
                            generate_new_equipment(
                                item,
                                &ctx.accounts.game,
                                &ctx.accounts.player,
                                spell_book_account.level,
                                Some(rarity),
                                &mut rand,
                            )?;
                            burn_item_account = false;
                        }
                        _ => {}
                    }
                }
            }

            caster.modifiers.spell_book = None;

            zombify_account(
                &mut spell_book_account,
                ctx.accounts.authority.to_account_info(),
                ctx.program_id,
            )?;

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
