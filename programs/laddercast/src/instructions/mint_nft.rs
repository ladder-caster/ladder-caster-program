use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_token_metadata::instruction::create_metadata_accounts_v2;
use mpl_token_metadata::state::Creator;
use spl_token::instruction::AuthorityType;

use crate::account::{Caster, Game, Item, MerkleRootNFT, MetadataCaster, MetadataItem, MetadataNFTCaster, MetadataNFTItem, Player};
use crate::error::ErrorCode;
use crate::utils::{EXPERIENCE_REQUIRED_PER_LEVEL, get_merkle_string_for_caster, get_merkle_string_for_item, get_name_for_mint, ItemType, MetaplexTokenMetadata, NFT_CASTER_NAME, NFT_CREATOR_SPLITTER_PUBKEY, NFT_MINT_DESCRIPTION, verify_merkle_proof};

#[derive(Accounts)]
#[instruction(item_type_str: String, item_level: u8)]
pub struct MintItem<'info> {
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub game: Box<Account<'info, Game>>,

    #[account(mut, seeds = [b"game_signer"], bump)]
    pub game_signer: UncheckedAccount<'info>,

    #[account(mut, has_one = authority, has_one = game)]
    pub player: Account<'info, Player>,
    #[account(mut,
    close = authority,
    constraint = item.game == game.key(),
    constraint = item.owner == player.key(),
    constraint = item.equipped_owner == None
    )]
    pub item: Box<Account<'info, Item>>,

    #[account(mut,
    seeds = [b"merkle_roots", game.key().as_ref(), item_type_str.as_bytes(), item_level.to_string().as_ref()],
    bump = merkle_root_nft.bump)]
    pub merkle_root_nft: Box<Account<'info, MerkleRootNFT>>,

    #[account(mut)]
    pub metaplex_metadata_account: UncheckedAccount<'info>,
    // Where you write the stuff
    pub metaplex_token_metadata_program: Program<'info, MetaplexTokenMetadata>, //Used to write to the ^

    #[account(init,
    mint::decimals = 0,
    mint::authority = authority,
    payer = authority)]
    pub nft_mint: Account<'info, Mint>,
    #[account(init,
    associated_token::mint = nft_mint,
    associated_token::authority = authority,
    payer = authority)]
    pub nft_token: Account<'info, TokenAccount>,
    #[account(init,
    seeds = [b"metadata".as_ref(), nft_mint.key().as_ref()],
    bump,
    payer = authority,
    space = MetadataNFTItem::SIZE
    )]
    pub nft_metadata: Box<Account<'info, MetadataNFTItem>>,
}

pub fn mint_item(
    ctx: Context<MintItem>,
    nft_uri: String,
    merkle_proof: Vec<[u8; 32]>,
    item_type_str: String,
    item_level: u8,
) -> ProgramResult {
    let item = **ctx.accounts.item.clone();

    if item.equipped_owner != None {
        return Err(ErrorCode::ItemCantBeMintIfEquipped.into());
    }

    let provided_item_type_str = item.item_type.to_string();
    let provided_item_level = if provided_item_type_str == "combined" || provided_item_type_str == "spellBook" { 0 } else { item.level };

    if provided_item_type_str != item_type_str || provided_item_level != item_level {
        return Err(ErrorCode::InvalidMerkleRootSent.into());
    }

    //Verify that the provided uri is the valid one
    let merkle_string = get_merkle_string_for_item(&nft_uri, item);

    if merkle_string == None {
        return Err(ErrorCode::InvalidItemForMerkleProof.into());
    }

    let merkle_node = anchor_lang::solana_program::keccak::hash(merkle_string.unwrap().as_ref());

    if !verify_merkle_proof(
        merkle_proof,
        ctx.accounts.merkle_root_nft.merkle_root_nft,
        merkle_node.0,
    ) {
        return Err(ErrorCode::InvalidNFTURI.into());
    };

    ctx.accounts.nft_metadata.item = MetadataItem {
        game: item.game,
        owner: item.owner,
        level: item.level,
        item_type: item.item_type.clone(),
        equipped_owner: None,
    };

    ctx.accounts.nft_metadata.self_bump = *ctx.bumps.get("nft_metadata").unwrap();
    ctx.accounts.nft_metadata.mint = ctx.accounts.nft_mint.key();

    // create metaplex token metadata
    let metadata_infos = vec![
        ctx.accounts.authority.to_account_info(),
        ctx.accounts.metaplex_metadata_account.to_account_info(),
        ctx.accounts.nft_mint.to_account_info(),
        ctx.accounts.game_signer.to_account_info(),
        ctx.accounts
            .metaplex_token_metadata_program
            .to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.rent.to_account_info(),
    ];

    let name = get_name_for_mint(&ctx.accounts.item.item_type);

    if name == None {
        return Err(ErrorCode::InvalidItemType.into());
    }

    let seeds = &[b"game_signer".as_ref(), &[ctx.accounts.game.signer_bump]];

    let signer = &[&seeds[..]];

    anchor_lang::solana_program::program::invoke_signed(
        &create_metadata_accounts_v2(
            *ctx.accounts.metaplex_token_metadata_program.key,
            *ctx.accounts.metaplex_metadata_account.key,
            ctx.accounts.nft_mint.to_account_info().key(),
            *ctx.accounts.authority.key,
            *ctx.accounts.authority.key,
            *ctx.accounts.game_signer.key,
            name.unwrap().to_string(),
            NFT_MINT_DESCRIPTION.to_string(),
            nft_uri,
            Some(vec![
                Creator {
                    address: *ctx.accounts.game_signer.key,
                    verified: true,
                    share: 0,
                },
                Creator {
                    address: Pubkey::from_str(NFT_CREATOR_SPLITTER_PUBKEY).unwrap(),
                    verified: false,
                    share: 100,
                },
            ]),
            100, //1% seller fee
            true,
            true,
            None,
            None,
        ),
        &metadata_infos[..],
        signer,
    )?;

    // We got to mint to your newly created wallet
    token::mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            token::MintTo {
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.nft_token.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        1,
    )?;

    token::set_authority(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            token::SetAuthority {
                account_or_mint: ctx.accounts.nft_mint.to_account_info().clone(),
                current_authority: ctx.accounts.authority.to_account_info().clone(),
            },
        ),
        AuthorityType::MintTokens,
        None,
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(item_level: u8)]
pub struct MintCaster<'info> {
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub game: Box<Account<'info, Game>>,

    #[account(mut, seeds = [b"game_signer"], bump)]
    pub game_signer: UncheckedAccount<'info>,

    #[account(mut,
    seeds = [b"merkle_roots", game.key().as_ref(), b"combined", item_level.to_string().as_bytes()],
    bump = merkle_root_nft.bump)]
    pub merkle_root_nft: Box<Account<'info, MerkleRootNFT>>,

    #[account(mut, has_one = authority, has_one = game)]
    pub player: Box<Account<'info, Player>>,
    #[account(mut,
    close = authority,
    constraint = caster.owner == player.key()
    )]
    pub caster: Box<Account<'info, Caster>>,

    #[account(mut)]
    // Where you write the stuff
    pub metaplex_metadata_account: UncheckedAccount<'info>,
    pub metaplex_token_metadata_program: Program<'info, MetaplexTokenMetadata>, //Used to write to the ^

    #[account(init,
    mint::decimals = 0,
    mint::authority = authority,
    payer = authority)]
    pub nft_mint: Account<'info, Mint>,
    #[account(init,
    associated_token::mint = nft_mint,
    associated_token::authority = authority,
    payer = authority)]
    pub nft_token: Account<'info, TokenAccount>,
    #[account(init,
    seeds = [b"metadata".as_ref(), nft_mint.key().as_ref()],
    bump,
    payer = authority,
    space = MetadataNFTCaster::SIZE
    )]
    pub nft_metadata: Box<Account<'info, MetadataNFTCaster>>,
}

pub fn mint_caster(
    ctx: Context<MintCaster>,
    nft_uri: String,
    merkle_proof: Vec<[u8; 32]>,
    item_level: u8,
) -> ProgramResult {
    let mut caster = **ctx.accounts.caster.clone();

    //To prevent infinite number of nft metadata, we set the experience to the min for the current level
    if caster.level == 1 {
        caster.experience = 0;
    } else {
        caster.experience = EXPERIENCE_REQUIRED_PER_LEVEL[(caster.level - 2) as usize]; //-1 since 0 based array
    }


    if caster.turn_commit != None {
        return Err(ErrorCode::InvalidCasterMintPendingTurn.into());
    }

    if [
        caster.modifiers.robe,
        caster.modifiers.staff,
        caster.modifiers.head,
        caster.modifiers.spell_book,
    ]
        .iter()
        .any(|item_pub| *item_pub != None)
    {
        return Err(ErrorCode::InvalidCasterMintEquipped.into());
    }

    if item_level != 0 {
        return Err(ErrorCode::InvalidMerkleRootSent.into());
    }

    //Verify that the provided uri is the valid one
    let merkle_string = get_merkle_string_for_caster(&nft_uri, caster);

    let merkle_node = anchor_lang::solana_program::keccak::hash(merkle_string.as_ref());

    if !verify_merkle_proof(
        merkle_proof,
        ctx.accounts.merkle_root_nft.merkle_root_nft,
        merkle_node.0,
    ) {
        return Err(ErrorCode::InvalidNFTURI.into());
    };

    ctx.accounts.nft_metadata.caster = MetadataCaster {
        version: caster.version,
        level: caster.level,
        experience: caster.experience,
        owner: caster.owner,
        modifiers: caster.modifiers.clone(),
        turn_commit: None,
    };

    ctx.accounts.nft_metadata.self_bump = *ctx.bumps.get("nft_metadata").unwrap();
    ctx.accounts.nft_metadata.mint = ctx.accounts.nft_mint.key();

    // create metaplex token metadata
    let metadata_infos = vec![
        ctx.accounts.authority.to_account_info(),
        ctx.accounts.metaplex_metadata_account.to_account_info(),
        ctx.accounts.nft_mint.to_account_info(),
        ctx.accounts.game_signer.to_account_info(),
        ctx.accounts
            .metaplex_token_metadata_program
            .to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.rent.to_account_info(),
    ];

    let seeds = &[b"game_signer".as_ref(), &[ctx.accounts.game.signer_bump]];

    let signer = &[&seeds[..]];

    anchor_lang::solana_program::program::invoke_signed(
        &create_metadata_accounts_v2(
            *ctx.accounts.metaplex_token_metadata_program.key,
            *ctx.accounts.metaplex_metadata_account.key,
            ctx.accounts.nft_mint.to_account_info().key(),
            *ctx.accounts.authority.key,
            *ctx.accounts.authority.key,
            *ctx.accounts.game_signer.key,
            NFT_CASTER_NAME.to_string(),
            NFT_MINT_DESCRIPTION.to_string(),
            nft_uri,
            Some(vec![
                Creator {
                    address: *ctx.accounts.game_signer.key,
                    verified: true,
                    share: 0,
                },
                Creator {
                    address: Pubkey::from_str(NFT_CREATOR_SPLITTER_PUBKEY).unwrap(),
                    verified: false,
                    share: 100,
                },
            ]),
            100, //1% seller fee
            true,
            true,
            None,
            None,
        ),
        &metadata_infos[..],
        signer,
    )?;

    // We got to mint to your newly created wallet
    token::mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            token::MintTo {
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.nft_token.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        1,
    )?;

    token::set_authority(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            token::SetAuthority {
                account_or_mint: ctx.accounts.nft_mint.to_account_info().clone(),
                current_authority: ctx.accounts.authority.to_account_info().clone(),
            },
        ),
        AuthorityType::MintTokens,
        None,
    )?;

    Ok(())
}
