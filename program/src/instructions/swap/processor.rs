use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult};

use crate::{cpi::dlmm::Swap2 as MeteoraSwap2};
use crate::instructions::{
    SwapAccounts,
    data::SwapData
};

pub struct Swap<'a> {
    pub accounts: SwapAccounts<'a>,
    pub data: SwapData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Swap<'a> {
    type Error = ProgramError;

    fn try_from(
        (raw_data, accounts): (&'a [u8], &'a [AccountInfo]),
    ) -> Result<Self, Self::Error> {
        let data = SwapData::try_from(raw_data)?;
        let accounts = SwapAccounts::try_from_slice(accounts, data.num_bin_arrays as usize)?;
        Ok(Self { accounts, data })
    }
}

impl Swap<'_> {
    fn process(&self) -> ProgramResult {
        MeteoraSwap2 {
            lb_pair: self.accounts.lb_pair,
            bitmap_ext:      self.accounts.bitmap_ext,
            reserve_x:       self.accounts.reserve_x ,
            reserve_y:       self.accounts.reserve_y ,
            user_token_in:   self.accounts.token_in ,
            user_token_out:  self.accounts.token_out ,
            token_x_mint:    self.accounts.mint_x ,
            token_y_mint:    self.accounts.mint_y ,
            oracle:          self.accounts.oracle ,
            host_fee_in:     self.accounts.host_fee,
            user:            self.accounts.user,
            token_x_program: self.accounts.token_prog_x ,
            token_y_program: self.accounts.token_prog_y ,
            memo_program:    self.accounts.memo_program,
            event_authority: self.accounts.event_auth ,
            program:         self.accounts.dlmm_program,
            bin_arrays:      self.accounts.bin_arrays ,
            amount_in:       self.data.amount_in,
            min_amount_out:  self.data.min_out ,
        }.invoke()?;
        Ok(())
    }
}

pub fn process_swap(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Swap::try_from((instruction_data, accounts))?.process()
}
