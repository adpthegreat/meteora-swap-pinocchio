use pinocchio::program_error::ProgramError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MeteoraSwapError {
    InvalidInstructionData = 0,

    InstructionSysvarTooShort = 1,

    WrongProgramId = 2,

    WrongDiscriminator = 3,

    ArithmeticOverflow = 4,
}

impl From<MeteoraSwapError> for ProgramError {
    fn from(e: MeteoraSwapError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
