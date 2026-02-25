use crate::{write_bytes, UNINIT_BYTE};

//helpers for serializing accounts info into the dlmm data for token 2022 accounts, empty by defualt

// https://github.com/MeteoraAg/dlmm-sdk/blob/40f72d72419cd0b677910f65fe4e6595cf935911/market_making/src/core.rs#L431

//https://github.com/MeteoraAg/dlmm-sdk/blob/main/idls/dlmm.json#L7681 IDL reference for the types

const BASE_DATA_LEN: usize = 28; // discriminator + amounts + remaining_accounts_info


#[derive(Clone, Debug, PartialEq)]
pub enum AccountsType {
    TransferHookX,
    TransferHookY,
    TransferHookReward,
    TransferHookMultiReward(u8),
}

#[derive(Clone, Debug, PartialEq)]
pub struct RemainingAccountsSlice {
    pub accounts_type: AccountsType,
    pub length: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RemainingAccountsInfo {
    pub slices: Vec<RemainingAccountsSlice>,
}

// Calculate actual length based on remaining accounts
fn calculate_data_len(remaining_accounts_info: &RemainingAccountsInfo) -> usize {
    let mut len = BASE_DATA_LEN;
    
    // Vec length prefix (4 bytes)
    // Already included in BASE_DATA_LEN
    
    // Each slice: 1-2 bytes (enum) + 1 byte (length)
    for slice in &remaining_accounts_info.slices {
        len += match slice.accounts_type {
            AccountsType::TransferHookMultiReward(_) => 3, // 2 byte enum + 1 byte length
            _ => 2, // 1 byte enum + 1 byte length
        };
    }
    
    len
}

// Serialize remaining_accounts_info
fn serialize_remaining_accounts_info(
    dest: &mut [MaybeUninit<u8>],
    info: &RemainingAccountsInfo,
) -> usize {
    let mut offset = 0;
    
    // Write vec length (u32 little-endian)
    let vec_len = info.slices.len() as u32;
    write_bytes(&mut dest[offset..offset+4], &vec_len.to_le_bytes());
    offset += 4;
    
    // Write each slice
    for slice in &info.slices {
        // Write enum discriminant and optional data
        match slice.accounts_type {
            AccountsType::TransferHookX => {
                dest[offset].write(0u8);
                offset += 1;
            }
            AccountsType::TransferHookY => {
                dest[offset].write(1u8);
                offset += 1;
            }
            AccountsType::TransferHookReward => {
                dest[offset].write(2u8);
                offset += 1;
            }
            AccountsType::TransferHookMultiReward(n) => {
                dest[offset].write(3u8);
                dest[offset + 1].write(n);
                offset += 2;
            }
        }
        
        // Write length field
        dest[offset].write(slice.length);
        offset += 1;
    }
    
    offset
}

// USAGE
// Complete instruction serialization
pub fn serialize_swap_instruction(
    amount_in: u64,
    min_amount_out: u64,
    remaining_accounts_info: RemainingAccountsInfo,
) -> Vec<u8> {
    let data_len = calculate_data_len(&remaining_accounts_info);
    let mut instruction_data = vec![UNINIT_BYTE; data_len];
    
    // Write discriminator
    write_bytes(&mut instruction_data[0..8], &DISCRIMINATOR);
    
    // Write amount_in
    write_bytes(&mut instruction_data[8..16], &amount_in.to_le_bytes());
    
    // Write min_amount_out
    write_bytes(&mut instruction_data[16..24], &min_amount_out.to_le_bytes());
    
    // Write remaining_accounts_info (this is what you were missing!)
    serialize_remaining_accounts_info(
        &mut instruction_data[24..],
        &remaining_accounts_info,
    );
    
    // Convert MaybeUninit to initialized bytes
    unsafe {
        std::mem::transmute::<Vec<MaybeUninit<u8>>, Vec<u8>>(instruction_data)
    }
}