use anchor_lang::prelude::*;

use crate::account::{Caster, Game, Item, Player};
use crate::error::ErrorCode;
use crate::utils::GAME_CREATOR_AUTHORITY_PUBKEY;
use crate::utils::{EquipmentType, ItemType};

#[derive(Accounts)]
pub struct FixRedeemSpell<'info> {
  pub authority: Signer<'info>,
  #[account(mut)]
  pub caster: Account<'info, Caster>,
}

pub fn fix_redeem_spell(ctx: Context<FixRedeemSpell>) -> ProgramResult {
  let caster = &mut ctx.accounts.caster;

  if ctx.accounts.authority.key().to_string() != GAME_CREATOR_AUTHORITY_PUBKEY {
    return Err(ErrorCode::NotSuperAdmin.into());
  }

  if caster.modifiers.spell_book != None {
    caster.modifiers.spell_book = None;
  }
  Ok(())
}
