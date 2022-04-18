use anchor_lang::prelude::{AccountInfo, ProgramResult, Pubkey};
use anchor_lang::solana_program::serialize_utils::{read_pubkey, read_u16};
use anchor_lang::solana_program::sysvar::instructions::load_current_index_checked;

use crate::error::ErrorCode;
use crate::msg;
use crate::utils::{ActionType, TurnCommit};

fn validate_program_ids(
    instruction_sysvar_acc_info: &AccountInfo,
    ctx_program_id: &Pubkey,
) -> bool {
    let instruction_sysvar = instruction_sysvar_acc_info.data.borrow();

    let mut idx = 0;
    let num_instructions = read_u16(&mut idx, &instruction_sysvar).unwrap();

    //Validate program ids are only the ones that are accepted
    for index in 0..num_instructions {
        let mut current = 2 + (index * 2) as usize;
        let start = read_u16(&mut current, &instruction_sysvar).unwrap();

        current = start as usize;
        let num_accounts = read_u16(&mut current, &instruction_sysvar).unwrap();
        current += (num_accounts as usize) * (1 + 32);
        let program_id = read_pubkey(&mut current, &instruction_sysvar).unwrap();

        if program_id != *ctx_program_id {
            return false;
        }
    }

    true
}

//Take from https://github.com/metaplex-foundation/metaplex-program-library/blob/master/candy-machine/program/src/lib.rs#L409
pub fn validate_is_last_instructions_and_program_ids(
    instruction_sysvar_acc_info: &AccountInfo,
    ctx_program_id: &Pubkey,
) -> ProgramResult {
    let instruction_sysvar = instruction_sysvar_acc_info.data.borrow();

    if !validate_program_ids(&instruction_sysvar_acc_info, ctx_program_id) {
        return Err(ErrorCode::InvalidInstructionOrdering.into());
    }

    //Validates that the instruction is the last one
    let current_ins_index = load_current_index_checked(instruction_sysvar_acc_info)
        .map_err(|_| ErrorCode::InvalidInstructionOrdering)?;

    let last_index = instruction_sysvar.len() - 2;

    let mut last_index_data: [u8; 2] = [0; 2];
    last_index_data.copy_from_slice(&instruction_sysvar[last_index..last_index + 2]);

    if u16::from_le_bytes(last_index_data) != current_ins_index {
        return Err(ErrorCode::InvalidInstructionOrdering.into());
    }

    Ok(())
}

pub fn validate_instruction_order_and_program_ids(
    instruction_sysvar_acc_info: &AccountInfo,
    ctx_program_id: &Pubkey,
    turn_commit: &Option<TurnCommit>,
    action_type: ActionType,
) -> ProgramResult {
    let instruction_sysvar = instruction_sysvar_acc_info.data.borrow();

    if !validate_program_ids(&instruction_sysvar_acc_info, ctx_program_id) {
        return Err(ErrorCode::InvalidInstructionOrdering.into());
    }

    let action_order_index: u16;

    match turn_commit {
        None => return Err(ErrorCode::EmptyTurnCommit.into()),
        Some(_) => match action_type {
            ActionType::Loot => action_order_index = 0,
            ActionType::Spell => action_order_index = 1,
            ActionType::Move => action_order_index = 2,
            ActionType::Craft => action_order_index = 3,
            ActionType::Reward => action_order_index = u16::MAX
        },
    }

    //Validates that the instruction is the last one
    let current_ins_index = load_current_index_checked(instruction_sysvar_acc_info)
        .map_err(|_| ErrorCode::InvalidInstructionOrdering)?;

    if action_order_index != u16::MAX {
        let next_action_order_index = turn_commit.unwrap().actions.get_highest_value() + 1;

        let last_index = instruction_sysvar.len() - 2;

        let mut last_index_data: [u8; 2] = [0; 2];
        last_index_data.copy_from_slice(&instruction_sysvar[last_index..last_index + 2]);

        if next_action_order_index as u16 != current_ins_index && u16::from_le_bytes(last_index_data) != current_ins_index {
            msg!("last index: {:?}, current {:?}", u16::from_le_bytes(last_index_data), current_ins_index);
            return Err(ErrorCode::InvalidInstructionOrdering.into());
        }
    } else {
        let last_index = instruction_sysvar.len() - 2;

        let mut last_index_data: [u8; 2] = [0; 2];
        last_index_data.copy_from_slice(&instruction_sysvar[last_index..last_index + 2]);

        if u16::from_le_bytes(last_index_data) != current_ins_index {
            return Err(ErrorCode::InvalidInstructionOrdering.into());
        }
    }

    Ok(())
}
