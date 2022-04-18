use anchor_lang::prelude::*;

use crate::account::*;
use crate::error::ErrorCode;
use crate::utils::GAME_CREATOR_AUTHORITY_PUBKEY;

#[derive(Accounts)]
#[instruction(item_type_str: String, item_level: u8)]
pub struct UpdateMerkleRoot<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub game_account: Box<Account<'info, Game>>,

    #[account(init_if_needed,
    seeds = [b"merkle_roots", game_account.key().as_ref(), item_type_str.as_bytes(), item_level.to_string().as_bytes()],
    bump,
    payer = authority,
    space = MerkleRootNFT::SIZE
    )]
    pub merkle_root_nft: Account<'info, MerkleRootNFT>,
}

pub fn update_merkle_root(
    ctx: Context<UpdateMerkleRoot>,
    merkle_root_nft: [u8; 32],
    _item_type_str: String,
    _item_level: u8,
) -> ProgramResult {
    // if ctx.accounts.authority.key().to_string() != GAME_CREATOR_AUTHORITY_PUBKEY {
    //     return Err(ErrorCode::NotSuperAdmin.into());
    // }

    let merkle_root_nft_acc = &mut ctx.accounts.merkle_root_nft;

    merkle_root_nft_acc.bump = *ctx.bumps.get("merkle_root_nft").unwrap();

    merkle_root_nft_acc.merkle_root_nft = merkle_root_nft;

    Ok(())
}
