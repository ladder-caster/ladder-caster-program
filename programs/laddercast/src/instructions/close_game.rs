use anchor_lang::prelude::*;

use crate::account::*;
use crate::error::ErrorCode;
use crate::utils::constants::*;

#[derive(Accounts)]
pub struct CloseGame<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(mut)]
    pub game_account: Box<Account<'info, Game>>,
}

pub fn close_game(ctx: Context<CloseGame>) -> ProgramResult {
    if ctx.accounts.authority.key().to_string() != GAME_CREATOR_AUTHORITY_PUBKEY {
        return Err(ErrorCode::NotSuperAdmin.into());
    }

    let game = &mut ctx.accounts.game_account;
    let authority = ctx.accounts.authority.to_account_info();

    // Take that money
    let dest_starting_lamports = authority.lamports();
    **authority.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(game.to_account_info().lamports())
        .unwrap();
    **game.to_account_info().lamports.borrow_mut() = 0;

    // Exit and write
    game.exit(ctx.program_id)?;

    Ok(())
}
