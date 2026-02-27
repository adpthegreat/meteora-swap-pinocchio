use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

// Processor for the `SwapA` instruction (discriminator 0).
//
// ### Accounts:
//   0.  `[SIGNER]`  user
//   1.  `[WRITE]`   lb_pair 
//   2.  `[WRITE]`   bin_array_bitmap_ext 
//   3.  `[WRITE]`   reserve_x 
//   4.  `[WRITE]`   reserve_y 
//   5.  `[WRITE]`   user_token_in 
//   6.  `[WRITE]`   user_token_out 
//   7.  `[]`        token_x_mint 
//   8.  `[]`        token_y_mint 
//   9.  `[WRITE]`   oracle 
//   10. `[WRITE]`   host_fee_in 
//   11. `[]`        token_x_program 
//   12. `[]`        token_y_program 
//   13. `[]`        memo_program
//   14. `[]`        event_authority 
//   15. `[]`        dlmm_program
//   +N  `[WRITE]`   bin arrays for swap A  (N = num_bin_arrays )
//
// ### Instruction data (after discriminator):
//   [0..8]    amount_in        u64 le
//   [8..16]   min_out          u64 le
//   [16]      num_bin_arrays   u8

pub struct SwapAccounts<'a> {
    pub user:            &'a AccountInfo,
    pub lb_pair :       &'a AccountInfo,
    pub bitmap_ext :    &'a AccountInfo,
    pub reserve_x :     &'a AccountInfo,
    pub reserve_y :     &'a AccountInfo,
    pub token_in :      &'a AccountInfo,
    pub token_out :     &'a AccountInfo,
    pub mint_x :        &'a AccountInfo,
    pub mint_y :        &'a AccountInfo,
    pub oracle :        &'a AccountInfo,
    pub host_fee :      &'a AccountInfo,
    pub token_prog_x :  &'a AccountInfo,
    pub token_prog_y :  &'a AccountInfo,
    pub memo_program:    &'a AccountInfo,
    pub event_auth :    &'a AccountInfo,
    pub dlmm_program: &'a AccountInfo,
    // variable remaining accounts
    pub bin_arrays :    &'a [AccountInfo],
}

impl<'a> SwapAccounts<'a> {
    pub fn try_from_slice(
        accounts: &'a [AccountInfo],
        num_bin_arrays : usize,
    ) -> Result<Self, ProgramError> {
        let [
            user,
            lb_pair , bitmap_ext , reserve_x , reserve_y ,
            token_in , token_out , mint_x , mint_y ,
            oracle , host_fee , token_prog_x , token_prog_y ,
            memo_program, event_auth, dlmm_program,
            remaining @ ..
        ] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if remaining.len() < num_bin_arrays  {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let bin_arrays = &remaining[..num_bin_arrays];

        Ok(Self {
            user,
            lb_pair, bitmap_ext, reserve_x, reserve_y,
            token_in, token_out, mint_x, mint_y,
            oracle, host_fee, token_prog_x, token_prog_y,
            memo_program, event_auth, dlmm_program,
            bin_arrays
        })
    }
}
