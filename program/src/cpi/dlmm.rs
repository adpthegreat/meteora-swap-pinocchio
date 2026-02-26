use core::{mem::MaybeUninit, slice::from_raw_parts};
use core::mem;

use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Signer},
    program::invoke_signed,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{write_bytes, UNINIT_BYTE};

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
const MAX_ACCOUNTS:   usize = FIXED_ACCOUNTS + MAX_BIN_ARRAYS;

pub struct Swap2<'a> {
    pub lb_pair:                    &'a AccountInfo,
    pub bitmap_ext: &'a AccountInfo,
    pub reserve_x:                  &'a AccountInfo,
    pub reserve_y:                  &'a AccountInfo,
    pub user_token_in:              &'a AccountInfo,
    pub user_token_out:             &'a AccountInfo,
    pub token_x_mint:               &'a AccountInfo,
    pub token_y_mint:               &'a AccountInfo,
    pub oracle:                     &'a AccountInfo,
    pub host_fee_in:                &'a AccountInfo,
    pub user:                       &'a AccountInfo,
    pub token_x_program:            &'a AccountInfo,
    pub token_y_program:            &'a AccountInfo,
    pub memo_program:               &'a AccountInfo,
    pub event_authority:            &'a AccountInfo,
    pub program:                    &'a AccountInfo,
    /// Remaining bin-array accounts (up to MAX_BIN_ARRAYS = 8).
    pub bin_arrays:                 &'a [AccountInfo],
    /// Instruction parameters.
    pub amount_in:                  u64,
    pub min_amount_out:             u64,
}

impl Swap2<'_> {
    #[inline(always)]
    pub fn invoke(&self) -> ProgramResult {
        self.invoke_signed(&[])
    }

    pub fn invoke_signed(&self, signers: &[Signer]) -> ProgramResult {
        let num_bin_arrays = self.bin_arrays.len();
        debug_assert!(num_bin_arrays <= MAX_BIN_ARRAYS, "too many bin arrays");
        // [0..8]   discriminator
        // [8..16]  amount_in         u64 le
        // [16..24] min_amount_out    u64 le
        // [24..30] RemainingAccountsInfo (borsh):
        // [u32 le = 1][type = 0x00 (BinArrays)][count: u8]
        const DATA_LEN: usize = 28;
        let mut instruction_data = [UNINIT_BYTE; DATA_LEN];

        write_bytes(&mut instruction_data[0..8],  &DISCRIMINATOR);
        write_bytes(&mut instruction_data[8..16], &self.amount_in.to_le_bytes());
        write_bytes(&mut instruction_data[16..24], &self.min_amount_out.to_le_bytes());
        write_bytes(&mut instruction_data[24..28], &0u32.to_le_bytes()); // vec len = 0
        //https://github.com/MeteoraAg/dlmm-sdk/blob/40f72d72419cd0b677910f65fe4e6595cf935911/market_making/src/core.rs#L431 
        //add remaining accounts infos if you have transfer hook (token2022) acounts

        //we don't use a Vec of AccountMetas because the Vec array allocations take up heap 
        //storage which results in more CUs

        //we can also use array_ref
        let total = FIXED_ACCOUNTS + num_bin_arrays;
        let mut metas: [MaybeUninit<AccountMeta>; MAX_ACCOUNTS] =
            unsafe { MaybeUninit::uninit().assume_init() };

        metas[0].write(AccountMeta::writable(self.lb_pair.key()));
        metas[1].write(AccountMeta::writable(self.bitmap_ext.key()));
        metas[2].write(AccountMeta::writable(self.reserve_x.key()));
        metas[3].write(AccountMeta::writable(self.reserve_y.key()));
        metas[4].write(AccountMeta::writable(self.user_token_in.key()));
        metas[5].write(AccountMeta::writable(self.user_token_out.key()));
        metas[6].write(AccountMeta::readonly(self.token_x_mint.key()));
        metas[7].write(AccountMeta::readonly(self.token_y_mint.key()));
        metas[8].write(AccountMeta::writable(self.oracle.key()));
        metas[9].write(if self.host_fee_in.key() == &ID {
            AccountMeta::readonly(self.program.key())
        } else { 
            AccountMeta::writable(self.host_fee_in.key())
        });
        metas[10].write(AccountMeta::readonly_signer(self.user.key()));
        metas[11].write(AccountMeta::readonly(self.token_x_program.key()));
        metas[12].write(AccountMeta::readonly(self.token_y_program.key()));
        metas[13].write(AccountMeta::readonly(self.memo_program.key()));
        metas[14].write(AccountMeta::readonly(self.event_authority.key()));
        metas[15].write(AccountMeta::readonly(self.program.key()));

        for (i, bin_array) in self.bin_arrays.iter().enumerate() {
            metas[FIXED_ACCOUNTS + i].write(AccountMeta::writable(bin_array.key()));
        }

        let mut infos: [MaybeUninit<&AccountInfo>; MAX_ACCOUNTS] =
            unsafe { MaybeUninit::uninit().assume_init() };

        infos[0].write(self.lb_pair);
        infos[1].write(self.bitmap_ext);
        infos[2].write(self.reserve_x);
        infos[3].write(self.reserve_y);
        infos[4].write(self.user_token_in);
        infos[5].write(self.user_token_out);
        infos[6].write(self.token_x_mint);
        infos[7].write(self.token_y_mint);
        infos[8].write(self.oracle);
        infos[9].write(self.host_fee_in);
        infos[10].write(self.user);
        infos[11].write(self.token_x_program);
        infos[12].write(self.token_y_program);
        infos[13].write(self.memo_program);
        infos[14].write(self.event_authority);
        infos[15].write(self.program);

        for (i, bin_array) in self.bin_arrays.iter().enumerate() {
            infos[FIXED_ACCOUNTS + i].write(bin_array);
        }

        // SAFETY: every slot is now initialised.
        // MaybeUninit<T> is guaranteed to have the same layout as T,
        // so transmuting [MaybeUninit<&AccountInfo>; N] → [&AccountInfo; N]
        // is defined behaviour once all slots are written.
        let infos: [&AccountInfo; MAX_ACCOUNTS] = unsafe { mem::transmute(infos) };
        
        let instruction = Instruction {
            program_id: &ID,
            accounts: unsafe {
                from_raw_parts(metas.as_ptr() as *const AccountMeta, total)
            },
            data: unsafe {
                from_raw_parts(instruction_data.as_ptr() as *const u8, DATA_LEN)
            },
        };

        invoke_signed(
            &instruction,
            &infos,
            signers,
        )
    }
}
