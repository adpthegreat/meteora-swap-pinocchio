use shank::ShankInstruction;

/// Instructions for meteoraswap. This is currently not used in the
/// program business logic, but we include it for IDL generation.

//DLMM swap 

/// Instructions for the MeteoraSwap program.
///
/// This enum is compiled only with `--features idl` and is never included
/// in the deployed binary. Its sole purpose is to let `shank idl` introspect
/// the account constraints and argument shapes and emit `idl/meteoraswap.json`,
/// which Codama then uses to generate JS and Rust clients.
#[repr(C, u8)]
#[derive(Clone, Debug, PartialEq, ShankInstruction)]
pub enum MeteoraSwapInstruction {
    #[account(0,  signer,   name = "user",                   desc = "Transaction signer and token owner")]
    #[account(1,  writable, name = "lb_pair",              desc = "DLMM lb_pair for swap")]
    #[account(2,  writable, name = "binrray_bitmap_ext", desc = "Bin array bitmap extension for swap (pass DLMM program ID if unused)")]
    #[account(3,  writable, name = "reserve_x",            desc = "Pool token X reserve for swap")]
    #[account(4,  writable, name = "reserve_y",            desc = "Pool token Y reserve for swap")]
    #[account(5,  writable, name = "user_token_in",        desc = "User ATA for swap input token")]
    #[account(6,  writable, name = "user_token_out",       desc = "User ATA for swap output token")]
    #[account(7,            name = "token_x_mint",         desc = "Token X mint for swap")]
    #[account(8,            name = "token_y_mint",         desc = "Token Y mint for swap")]
    #[account(9,  writable, name = "oracle",               desc = "Oracle account for swap")]
    #[account(10, writable, name = "host_fee_in",          desc = "Host fee account for swap A (pass DLMM program ID if unused)")]
    #[account(11,           name = "token_x_program",      desc = "SPL Token program for token X of swap")]
    #[account(12,           name = "token_y_program",      desc = "SPL Token program for token Y of swap")]
    #[account(13,           name = "memo_program",           desc = "Memo program")]
    #[account(14,           name = "event_authority",      desc = "DLMM event authority PDA for swap")]
    #[account(15,           name = "dlmm_program",           desc = "Meteora DLMM program")]
    Swap {
        /// Amount of input token to sell on pool
        amount_in: u64,
        /// Minimum output token to accept from pool
        min_out: u64,
        /// Number of bin-array remaining accounts 
        num_bin_arrays: u8,
    },

    /// Verify that a top-level DLMM `swap2` instruction exists at
    /// `expected_ix_index` in this transaction, optionally assert its pool is
    /// `expected_lb_pair`, then execute swap B via CPI.
    #[account(0,  signer,   name = "user",                   desc = "Transaction signer and token owner")]
    #[account(1,            name = "instructions_sysvar",    desc = "Instructions sysvar (Sysvar1nstructions1111...)")] //TODO move it down 
    #[account(2,  writable, name = "lb_pair",              desc = "DLMM lb_pair for swap")]
    #[account(3,  writable, name = "bin_array_bitmap_ext", desc = "Bin array bitmap extension for swap")]
    #[account(4,  writable, name = "reserve_x",            desc = "Pool token X reserve for swap")]
    #[account(5,  writable, name = "reserve_y",            desc = "Pool token Y reserve for swap")]
    #[account(6,  writable, name = "user_token_in",        desc = "User ATA for swap input token")]
    #[account(7,  writable, name = "user_token_out",       desc = "User ATA for swap output token")]
    #[account(8,            name = "token_x_mint",         desc = "Token X mint for swap")]
    #[account(9,            name = "token_y_mint",         desc = "Token Y mint for swap")]
    #[account(10, writable, name = "oracle",               desc = "Oracle account for swap")]
    #[account(11, writable, name = "host_fee_in",          desc = "Host fee account for swap")]
    #[account(12,           name = "token_x_program",      desc = "SPL Token program for token X of swap")]
    #[account(13,           name = "token_y_program",      desc = "SPL Token program for token Y of swap")]
    #[account(14,           name = "memo_program",           desc = "Memo program")]
    #[account(15,           name = "event_authority",      desc = "DLMM event authority PDA for swap")]
    #[account(16,           name = "dlmm_program",           desc = "Meteora DLMM program")]
    Swap2 {
        /// Amount of input token to sell on pool B.
        amount_in: u64,
        /// Minimum output token to accept from pool B.
        min_out: u64,
        /// Number of bin-array remaining accounts for swap B.
        num_bin_arrays: u8,
    },
}