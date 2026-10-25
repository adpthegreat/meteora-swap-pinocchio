#![no_std]

pub mod error;
// #[cfg(feature = "idl")]
pub mod instructions;
pub mod cpi;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

#[cfg(not(feature = "cpi-scratch"))]
use core::mem::MaybeUninit;
pinocchio_pubkey::declare_id!("7N3hMx1N2GQtYFohgrhD2ZDeqv3bnwTW4xXdz3kHmfXv");

#[cfg(not(feature = "cpi-scratch"))]
pub(crate) const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::<u8>::uninit();

#[cfg(not(feature = "cpi-scratch"))]
/// Write `source` bytes into a `MaybeUninit<u8>` destination slice.
#[inline(always)]
pub(crate) fn write_bytes(destination: &mut [MaybeUninit<u8>], source: &[u8]) {
    for (d, s) in destination.iter_mut().zip(source.iter()) {
        d.write(*s);
    }
}
