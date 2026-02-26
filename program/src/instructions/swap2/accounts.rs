use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
};

pub struct Swap2Accounts<'a> {
    pub user:         &'a AccountInfo,
    pub ix_sysvar:    &'a AccountInfo,
    pub lb_pair:    &'a AccountInfo,
    pub bitmap_ext: &'a AccountInfo,
    pub reserve_x:  &'a AccountInfo,
    pub reserve_y:  &'a AccountInfo,
    pub token_in:   &'a AccountInfo,
    pub token_out:  &'a AccountInfo,
    pub mint_x:     &'a AccountInfo,
    pub mint_y:     &'a AccountInfo,
    pub oracle:     &'a AccountInfo,
    pub host_fee:   &'a AccountInfo,
    pub token_prog_x: &'a AccountInfo,
    pub token_prog_y: &'a AccountInfo,
    pub memo_program: &'a AccountInfo,
    pub event_auth: &'a AccountInfo,
    pub dlmm_program: &'a AccountInfo,
    pub bin_arrays: &'a [AccountInfo],
}

impl<'a> Swap2Accounts<'a> {
    pub fn try_from_slice(
        accounts: &'a [AccountInfo],
        num_bin_arrays: usize,
    ) -> Result<Self, ProgramError> {
        let [
            user, ix_sysvar,
            lb_pair, bitmap_ext, reserve_x, reserve_y,
            token_in, token_out, mint_x, mint_y,
            oracle, host_fee, token_prog_x, token_prog_y,
            memo_program, event_auth, dlmm_program,
            remaining @ ..
        ] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if remaining.len() < num_bin_arrays {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let bin_arrays = &remaining[..num_bin_arrays];
        
        Ok(Self {
            user, ix_sysvar,
            lb_pair, bitmap_ext, reserve_x, reserve_y,
            token_in, token_out, mint_x, mint_y,
            oracle, host_fee, token_prog_x, token_prog_y,
            memo_program, event_auth, dlmm_program,
            bin_arrays,
        })
    }
}
