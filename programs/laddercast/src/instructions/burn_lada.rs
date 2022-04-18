use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::account::*;

#[derive(Accounts)]
pub struct BurnLada<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,

    pub game_account: Box<Account<'info, Game>>,

    #[account(mut, constraint = lada_mint.to_account_info().key() == game_account.lada_mint_account)]
    pub lada_mint: Account<'info, Mint>,

    #[account(mut)]
    pub lada_token_account: Account<'info, TokenAccount>,
}

//This function is to clear the distributed LADA during our testing phase
//Don't call this function or you will lose all your LADA
pub fn burn_lada(ctx: Context<BurnLada>) -> ProgramResult {
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            token::Burn {
                mint: ctx.accounts.lada_mint.to_account_info().clone(),
                to: ctx.accounts.lada_token_account.to_account_info().clone(), // STUPID NAME: its the token account you are burning the tokens from
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        ctx.accounts.lada_token_account.amount,
    )?;

    Ok(())
}
