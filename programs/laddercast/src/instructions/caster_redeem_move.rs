use anchor_lang::{prelude::*, solana_program::sysvar};

use crate::account::*;
use crate::error::ErrorCode;
use crate::utils::ACTION_MOVE_INDEX;

#[derive(Accounts)]
pub struct CasterRedeemMoveAction<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub game: Box<Account<'info, Game>>,

    #[account(mut, has_one = authority, has_one = game)]
    pub player: Box<Account<'info, Player>>,

    #[account(mut, constraint = caster.owner == player.key())]
    pub caster: Box<Account<'info, Caster>>,

    #[account(address = sysvar::instructions::id())]
    pub instruction_sysvar_account: UncheckedAccount<'info>,
}

pub fn caster_redeem_move<'info>(
    ctx: Context<'_, '_, '_, 'info, CasterRedeemMoveAction<'info>>,
) -> ProgramResult {
    let caster = &mut ctx.accounts.caster;
    let game = &ctx.accounts.game;

    match caster.turn_commit {
        None => {
            return Err(ErrorCode::EmptyTurnCommit.into());
        }
        Some(turn_commit) => {
            if game.turn_info.turn == turn_commit.turn {
                return Err(ErrorCode::SameTurnRedeem.into());
            }

            let index_next_action = turn_commit.actions.get_next_action_to_be_executed();

            if ACTION_MOVE_INDEX != index_next_action {
                return Err(ErrorCode::ActionOrderError.into());
            }

            caster.modifiers.tile_level = turn_commit.actions.mv.unwrap()[0];
            caster.modifiers.tile_column = turn_commit.actions.mv.unwrap()[1];

            //Set to max, since we filter to get the min to find next action
            caster.turn_commit.as_mut().unwrap().actions.action_order[index_next_action] = 0;
        }
    }

    Ok(())
}
