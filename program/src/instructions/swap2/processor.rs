
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::instructions::Instructions,
    ProgramResult,
};
use crate::{
    cpi::dlmm::{Swap2 as MeteoraSwap2, ID as DLMM_PROGRAM_ID, DISCRIMINATOR as SWAP2_DISCRIMINATOR},
    error::MeteoraSwapError, instructions::{
        Swap2Accounts, data::SwapData
    }
};

// Processor for the ` Swap2` instruction (discriminator 0x01).
//
// ### Accounts:
//   0.  `[SIGNER]`  user
//   1.  `[]`        instructions_sysvar  (Sysvar1nstructions1111...)
//   2.  `[WRITE]`   lb_pair
//   3.  `[WRITE]`   bin_array_bitmap_extension
//   4.  `[WRITE]`   reserve_x
//   5.  `[WRITE]`   reserve_y
//   6.  `[WRITE]`   user_token_in
//   7.  `[WRITE]`   user_token_out
//   8.  `[]`        token_x_mint
//   9.  `[]`        token_y_mint
//   10. `[WRITE]`   oracle
//   11. `[WRITE]`   host_fee_in
//   12. `[]`        token_x_program
//   13. `[]`        token_y_program
//   14. `[]`        memo_program
//   15. `[]`        event_authority
//   16. `[]`        dlmm_program
//   +N  `[WRITE]`   bin arrays for swap B  (N = numin_arrays)
//

pub struct Swap2<'a> {
    pub accounts: Swap2Accounts<'a>,
    pub data: SwapData, 
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Swap2<'a> {
    type Error = ProgramError;

    fn try_from(
        (raw_data, accounts): (&'a [u8], &'a [AccountInfo]),
    ) -> Result<Self, Self::Error> {
        let data     = SwapData::try_from(raw_data)?;
        let accounts = Swap2Accounts::try_from_slice(
            accounts,
            data.num_bin_arrays as usize,
        )?;
        Ok(Self { accounts, data })
    }
}

//Get the actual token amount received (not just quoted amount)
//Use that amount for the second swap
//basically the whole prupose of us introspecting is to get the actuall amount of USDC we received and use it for the swap 
//so we introspect the first ix, get the transfer ix to us, then use that as the input of the next swap

//a transaction that introspects the previous transaction to get the accurate balance of what the user received and not 
// the secondary quoted balance and uses that for the next swap

//we depend on diffing the balance with a quote and the actual balance

//create vault and then close after each txn - can we ddosed/griefing , unnecesary CUs 

// Verify that instruction #1 was indeed a Meteora swap
//we can't introspect internal instructions 
// Parse the token transfer that resulted from it
// Get the actual USDC amount received (not just quoted amount)
// Use that amount for the second swap

impl Swap2<'_> {
    fn process(&self) -> ProgramResult {
        let sysvar_data = self.accounts.ix_sysvar.try_borrow_data()?;

        let expected_ix_index = 0; 

        let num_instructions = unsafe { *(sysvar_data.as_ptr() as *const u16) };

        if expected_ix_index as usize >= num_instructions as usize {
            return Err(MeteoraSwapError::InstructionSysvarTooShort.into());
        }

        let ix_view = unsafe { Instructions::new_unchecked(sysvar_data) };

        let instruction = ix_view
            .load_instruction_at(expected_ix_index as usize)
            .map_err(|_| MeteoraSwapError::InstructionSysvarTooShort)?;

        if instruction.get_program_id() != &DLMM_PROGRAM_ID {
            return Err(MeteoraSwapError::WrongProgramId.into());
        }

        let ix_data = instruction.get_instruction_data();
        if ix_data.len() < 8 {
            return Err(MeteoraSwapError::WrongDiscriminator.into());
        }

        let actual_disc = unsafe { *(ix_data.as_ptr() as *const [u8; 8]) };

        if actual_disc == SWAP2_DISCRIMINATOR {
            return Err(MeteoraSwapError::WrongDiscriminator.into());
        }

        drop(ix_view);
        
        MeteoraSwap2 {
            lb_pair:                    self.accounts.lb_pair,
            bitmap_ext:                 self.accounts.bitmap_ext,
            reserve_x:                  self.accounts.reserve_x,
            reserve_y:                  self.accounts.reserve_y,
            user_token_in:              self.accounts.token_in,
            user_token_out:             self.accounts.token_out,
            token_x_mint:               self.accounts.mint_x,
            token_y_mint:               self.accounts.mint_y,
            oracle:                     self.accounts.oracle,
            host_fee_in:                self.accounts.host_fee,
            user:                       self.accounts.user,
            token_x_program:            self.accounts.token_prog_x,
            token_y_program:            self.accounts.token_prog_y,
            memo_program:               self.accounts.memo_program,
            event_authority:            self.accounts.event_auth,
            program:                    self.accounts.dlmm_program,
            bin_arrays:                 self.accounts.bin_arrays,
            amount_in:                  self.data.amount_in,
            min_amount_out:             self.data.min_out,
        }.invoke()
    }
}

pub fn process_swap2(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Swap2::try_from((instruction_data, accounts))?.process()
}
