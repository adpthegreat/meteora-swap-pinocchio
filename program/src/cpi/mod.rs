// Two interchangeable DLMM CPI builds, same `Swap2` API (see SOLANA.md):
// `MaybeUninit` buffers by default, `CpiScratch` with `--features cpi-scratch`.
#[cfg(not(feature = "cpi-scratch"))]
pub mod dlmm;

#[cfg(feature = "cpi-scratch")]
#[path = "dlmm_scratch.rs"]
pub mod dlmm;
#[cfg(feature = "cpi-scratch")]
pub mod scratch;

pub use dlmm::*;
