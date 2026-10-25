//! DLMM `swap2` CPI built on `CpiScratch` (`--features cpi-scratch`). Same
//! public API as the `MaybeUninit` version in `dlmm.rs`.

use pinocchio::{
    account_info::AccountInfo, instruction::Signer, program_error::ProgramError, pubkey::Pubkey,
    ProgramResult,
};

use super::scratch::{
    require_accounts, write_account, write_bytes, write_u32, write_u64, CpiBuffers,
    CpiInstruction, CpiScratch,
};

// LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo
pub const ID: Pubkey = [
   0x04, 0xE9, 0xE1, 0x2F, 0xBC, 0x84, 0xE8, 0x26, 0xC9, 0x32, 0xCC,
   0xE9, 0xE2, 0x64, 0x0C, 0xCE, 0x15, 0x59, 0x0C, 0x1C, 0x62, 0x73,
   0xB0, 0x92, 0x57, 0x08, 0xBA, 0x3B, 0x85, 0x20, 0xB0, 0xBC
];

/// swap2 discriminator: first 8 bytes of SHA256("global:swap2").
pub const DISCRIMINATOR: [u8; 8] = [0x41, 0x4B, 0x3F, 0x4C, 0xEB, 0x5B, 0x5B, 0x88];

const FIXED_ACCOUNTS: usize = 16;
const MAX_BIN_ARRAYS: usize = 8;
const MAX_ACCOUNTS: usize = FIXED_ACCOUNTS + MAX_BIN_ARRAYS;
/// discriminator + amount_in + min_amount_out + empty RemainingAccountsInfo vec
const DATA_LEN: usize = 28;

pub struct Swap2<'a> {
    pub lb_pair: &'a AccountInfo,
    pub bitmap_ext: &'a AccountInfo,
    pub reserve_x: &'a AccountInfo,
    pub reserve_y: &'a AccountInfo,
    pub user_token_in: &'a AccountInfo,
    pub user_token_out: &'a AccountInfo,
    pub token_x_mint: &'a AccountInfo,
    pub token_y_mint: &'a AccountInfo,
    pub oracle: &'a AccountInfo,
    pub host_fee_in: &'a AccountInfo,
    pub user: &'a AccountInfo,
    pub token_x_program: &'a AccountInfo,
    pub token_y_program: &'a AccountInfo,
    pub memo_program: &'a AccountInfo,
    pub event_authority: &'a AccountInfo,
    pub program: &'a AccountInfo,
    /// Remaining bin-array accounts (up to MAX_BIN_ARRAYS = 8).
    pub bin_arrays: &'a [AccountInfo],
    pub amount_in: u64,
    pub min_amount_out: u64,
}

impl Swap2<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let mut scratch = CpiScratch::<MAX_ACCOUNTS, DATA_LEN>::new(self.program);
        Swap2Cpi(self).invoke_signed((self.amount_in, self.min_amount_out), signers, &mut scratch)
    }
}

/// The typed CPI: owns DLMM `swap2`'s account order, flags and data layout.
struct Swap2Cpi<'r, 'a>(&'r Swap2<'a>);

impl<'a> CpiInstruction<'a> for Swap2Cpi<'_, 'a> {
    const PROGRAM_ID: Pubkey = ID;
    type Args = (u64, u64);

    fn write_accounts(&self, b: &mut CpiBuffers<'a, '_>) -> Result<usize, ProgramError> {
        let s = self.0;
        if s.bin_arrays.len() > MAX_BIN_ARRAYS {
            return Err(ProgramError::InvalidArgument);
        }
        let account_count = FIXED_ACCOUNTS + s.bin_arrays.len();
        require_accounts(b, account_count)?;
        write_account(b, 0, s.lb_pair, true, false);
        write_account(b, 1, s.bitmap_ext, true, false);
        write_account(b, 2, s.reserve_x, true, false);
        write_account(b, 3, s.reserve_y, true, false);
        write_account(b, 4, s.user_token_in, true, false);
        write_account(b, 5, s.user_token_out, true, false);
        write_account(b, 6, s.token_x_mint, false, false);
        write_account(b, 7, s.token_y_mint, false, false);
        write_account(b, 8, s.oracle, true, false);
        // host_fee_in: None is passed as the DLMM program id, which must stay read-only.
        write_account(b, 9, s.host_fee_in, s.host_fee_in.key() != &ID, false);
        write_account(b, 10, s.user, false, true);
        write_account(b, 11, s.token_x_program, false, false);
        write_account(b, 12, s.token_y_program, false, false);
        write_account(b, 13, s.memo_program, false, false);
        write_account(b, 14, s.event_authority, false, false);
        write_account(b, 15, s.program, false, false);
        for (index, bin_array) in (FIXED_ACCOUNTS..).zip(s.bin_arrays) {
            write_account(b, index, bin_array, true, false);
        }
        Ok(account_count)
    }

    fn write_data((amount_in, min_amount_out): (u64, u64), out: &mut [u8]) -> Result<usize, ProgramError> {
        let mut offset = 0;
        write_bytes(out, &mut offset, &DISCRIMINATOR)?;
        write_u64(out, &mut offset, amount_in)?;
        write_u64(out, &mut offset, min_amount_out)?;
        write_u32(out, &mut offset, 0)?; // RemainingAccountsInfo: empty vec
        Ok(offset)
    }
}
