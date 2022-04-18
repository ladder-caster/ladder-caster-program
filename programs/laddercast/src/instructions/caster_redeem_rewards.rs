use anchor_lang::{prelude::*, solana_program::sysvar};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};

use crate::account::*;
use crate::error::ErrorCode;
use crate::utils::{
    EARTH_INDEX, FIRE_INDEX, give_exp_to_caster_resources_burned, LADA_DISTRIBUTION_PER_TURN, WATER_INDEX,
};

#[derive(Accounts)]
pub struct CasterRedeemRewardAction<'info> {
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,

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

    #[account(mut, constraint = lada_mint_account.to_account_info().key() == game.lada_mint_account)]
    pub lada_mint_account: Box<Account<'info, Mint>>,

    #[account(mut, constraint = game_lada_token_account.key() == game.lada_token_account)]
    pub game_lada_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub lada_token_account: Box<Account<'info, TokenAccount>>,

    //Got to do -1 to the turn because we want the turn BEFORE the crank
    #[account(mut, seeds = [
    b"turn_data",
    game.to_account_info().key().as_ref(),
    (caster.turn_commit.unwrap().turn).to_string().as_ref()
    ], bump = game_turn_data.bump)]
    pub game_turn_data: Box<Account<'info, TurnData>>,

    #[account(address = sysvar::instructions::id())]
    pub instruction_sysvar_account: UncheckedAccount<'info>,
}

pub fn caster_redeem_reward<'info>(
    ctx: Context<'_, '_, '_, 'info, CasterRedeemRewardAction<'info>>,
) -> ProgramResult {
    let caster = &mut ctx.accounts.caster;
    let game = &ctx.accounts.game;
    let turn_data = &ctx.accounts.game_turn_data;

    match caster.turn_commit {
        None => {
            return Err(ErrorCode::EmptyTurnCommit.into());
        }
        Some(turn_commit) => {
            if game.turn_info.turn == turn_commit.turn {
                return Err(ErrorCode::SameTurnRedeem.into());
            }

            let index_next_action = turn_commit.actions.get_next_action_to_be_executed();

            if usize::MAX != index_next_action {
                return Err(ErrorCode::ActionOrderError.into());
            }

            //Give the experience to the caster based on burned resources
            give_exp_to_caster_resources_burned(
                caster,
                Some(turn_commit.resources_burned[FIRE_INDEX]),
                Some(turn_commit.resources_burned[EARTH_INDEX]),
                Some(turn_commit.resources_burned[WATER_INDEX]),
            );

            //Send LADA tokens based on proportion of resources burned by the user vs total resources
            let mut total_resources_burned_for_turn = turn_data.resource_1_burned as f64 + turn_data.resource_2_burned as f64 + turn_data.resource_3_burned as f64;
            let total_resources_burned_for_caster = turn_commit.resources_burned[FIRE_INDEX] as f64 + turn_commit.resources_burned[WATER_INDEX] as f64 + turn_commit.resources_burned[EARTH_INDEX] as f64;

            if total_resources_burned_for_turn == 0.0 {
                total_resources_burned_for_turn = 1.0;
            }

            let proportion_total = total_resources_burned_for_caster / total_resources_burned_for_turn;

            let amount: f64 = proportion_total * LADA_DISTRIBUTION_PER_TURN as f64;

            let cpi_accounts = Transfer {
                from: ctx
                    .accounts
                    .game_lada_token_account
                    .to_account_info()
                    .clone(),
                to: ctx.accounts.lada_token_account.to_account_info().clone(),
                authority: ctx.accounts.game_signer.to_account_info().clone(),
            };

            let transfer_cpi = CpiContext::new(
                ctx.accounts.token_program.to_account_info().clone(),
                cpi_accounts,
            );

            let seeds = &[b"game_signer".as_ref(), &[ctx.accounts.game.signer_bump]];
            let signer = &[&seeds[..]];

            token::transfer(transfer_cpi.with_signer(signer), amount as u64)?;

            //Reset caster's turn commit
            caster.turn_commit = None;
        }
    }

    Ok(())
}
