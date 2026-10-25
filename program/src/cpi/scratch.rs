//! `CpiScratch` CPI helpers (the Phoenix Rise pattern, see SOLANA.md). Built
//! with `--features cpi-scratch` to compare against the `MaybeUninit` path.

use core::array;

use pinocchio::{
    account_info::AccountInfo,
    cpi::invoke_signed_with_bounds,
    instruction::{AccountMeta, Instruction, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

/// Stack storage for one CPI. Every slot starts as a valid placeholder, so
/// no slot is ever uninitialised.
pub struct CpiScratch<'a, const MAX_ACCOUNTS: usize, const MAX_DATA_LEN: usize> {
    account_infos: [&'a AccountInfo; MAX_ACCOUNTS],
    account_metas: [AccountMeta<'a>; MAX_ACCOUNTS],
    data: [u8; MAX_DATA_LEN],
}

impl<'a, const MAX_ACCOUNTS: usize, const MAX_DATA_LEN: usize>
    CpiScratch<'a, MAX_ACCOUNTS, MAX_DATA_LEN>
{
    #[inline]
    pub fn new(fill_account: &'a AccountInfo) -> Self {
        Self {
            account_infos: array::from_fn(|_| fill_account),
            account_metas: array::from_fn(|_| AccountMeta::readonly(fill_account.key())),
            data: [0; MAX_DATA_LEN],
        }
    }

    #[inline]
    pub fn buffers(&mut self) -> CpiBuffers<'a, '_> {
        CpiBuffers {
            account_infos: &mut self.account_infos,
            account_metas: &mut self.account_metas,
            data: &mut self.data,
        }
    }
}

pub struct CpiBuffers<'a, 'b> {
    pub account_infos: &'b mut [&'a AccountInfo],
    pub account_metas: &'b mut [AccountMeta<'a>],
    pub data: &'b mut [u8],
}

#[inline]
pub fn require_accounts(buffers: &CpiBuffers<'_, '_>, account_count: usize) -> ProgramResult {
    if buffers.account_infos.len() < account_count || buffers.account_metas.len() < account_count
    {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    Ok(())
}

/// Unchecked: every `write_accounts` calls [`require_accounts`] with its
/// account count before writing, so `index` is always in bounds.
#[inline]
pub fn write_account<'a>(
    buffers: &mut CpiBuffers<'a, '_>,
    index: usize,
    account: &'a AccountInfo,
    is_writable: bool,
    is_signer: bool,
) {
    debug_assert!(index < buffers.account_infos.len() && index < buffers.account_metas.len());
    // SAFETY: callers bound `index` with `require_accounts`.
    unsafe {
        *buffers.account_infos.get_unchecked_mut(index) = account;
        *buffers.account_metas.get_unchecked_mut(index) =
            AccountMeta::new(account.key(), is_writable, is_signer);
    }
}

#[inline]
pub fn write_bytes(out: &mut [u8], offset: &mut usize, bytes: &[u8]) -> ProgramResult {
    let end = offset
        .checked_add(bytes.len())
        .ok_or(ProgramError::InvalidInstructionData)?;
    out.get_mut(*offset..end)
        .ok_or(ProgramError::InvalidInstructionData)?
        .copy_from_slice(bytes);
    *offset = end;
    Ok(())
}

#[inline]
pub fn write_u32(out: &mut [u8], offset: &mut usize, value: u32) -> ProgramResult {
    write_bytes(out, offset, &value.to_le_bytes())
}

#[inline]
pub fn write_u64(out: &mut [u8], offset: &mut usize, value: u64) -> ProgramResult {
    write_bytes(out, offset, &value.to_le_bytes())
}

/// One external instruction as a typed struct of account references. The
/// target program is a constant, so it can't be pointed at another program.
pub trait CpiInstruction<'a> {
    const PROGRAM_ID: Pubkey;

    type Args;

    fn write_accounts(&self, buffers: &mut CpiBuffers<'a, '_>) -> Result<usize, ProgramError>;

    fn write_data(args: Self::Args, out: &mut [u8]) -> Result<usize, ProgramError>;

    #[inline]
    fn invoke_signed<const MAX_ACCOUNTS: usize, const MAX_DATA_LEN: usize>(
        &self,
        args: Self::Args,
        signer_seeds: &[Signer<'_, '_>],
        scratch: &mut CpiScratch<'a, MAX_ACCOUNTS, MAX_DATA_LEN>,
    ) -> ProgramResult {
        let mut buffers = scratch.buffers();
        let account_count = self.write_accounts(&mut buffers)?;
        let data_len = Self::write_data(args, buffers.data)?;
        // `get` instead of indexing: an unprovable index keeps a panic path,
        // which links in ~5 KB of `core::fmt` (see SOLANA.md).
        let instruction = Instruction {
            program_id: &Self::PROGRAM_ID,
            accounts: buffers.account_metas.get(..account_count).ok_or(ProgramError::InvalidArgument)?,
            data: buffers.data.get(..data_len).ok_or(ProgramError::InvalidInstructionData)?,
        };
        let account_infos = buffers.account_infos.get(..account_count).ok_or(ProgramError::InvalidArgument)?;
        invoke_signed_with_bounds::<MAX_ACCOUNTS>(&instruction, account_infos, signer_seeds)
    }
}
