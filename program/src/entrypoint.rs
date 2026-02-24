use pinocchio::{
    account_info::AccountInfo, entrypoint, program_error::ProgramError, pubkey::Pubkey,
    ProgramResult,
};

use crate::instructions::{process_swap, process_swap2};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (discriminator, instruction_data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match discriminator {
        0 => process_swap(program_id, accounts, instruction_data),
        1 => process_swap2(program_id, accounts, instruction_data),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
