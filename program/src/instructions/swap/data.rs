use pinocchio::{
    program_error::ProgramError,
};
use crate::error::MeteoraSwapError;

pub struct SwapData {
    pub amount_in : u64,
    pub min_out : u64,
    pub num_bin_arrays: u8,
}

impl<'a> TryFrom<&'a [u8]> for SwapData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() < 17 {
            return Err(MeteoraSwapError::InvalidInstructionData.into());
        }
        Ok(Self {
            amount_in : u64::from_le_bytes(data[0..8].try_into().unwrap()),
            min_out : u64::from_le_bytes(data[8..16].try_into().unwrap()),
            num_bin_arrays :  data[16],
        })
    }
}
