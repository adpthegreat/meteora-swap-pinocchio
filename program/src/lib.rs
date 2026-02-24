#![no_std]

pub mod error;
// #[cfg(feature = "idl")]
pub mod instructions;
pub mod cpi;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

use core::mem::MaybeUninit;
// TODO update with your program ID
pinocchio_pubkey::declare_id!("2NCpU9nsgLfhqKX5CDVz24FfsZ8aRwjgUWtFbPsVZVu2");

/// Uninitialized byte sentinel – avoids zero-filling instruction data buffers.
pub(crate) const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::<u8>::uninit();

/// Write `source` bytes into a `MaybeUninit<u8>` destination slice.
#[inline(always)]
pub(crate) fn write_bytes(destination: &mut [MaybeUninit<u8>], source: &[u8]) {
    for (d, s) in destination.iter_mut().zip(source.iter()) {
        d.write(*s);
    }
}
