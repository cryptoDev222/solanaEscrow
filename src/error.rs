use thiserror::Error;
use solana_program::program_error::ProgramError;
use crate::{instruction::EscrowInstruction, error::EscrowError};

#[derive(Error, Debug, Copy, Clone)]
//macro for writing out errors without having to use fmt display
pub enum EscrowError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
}

impl From<EscrowError> for ProgramError {
    fn from(e: EscrowError) -> Self {
        //turns it into a number so that it can be called as an enum
        ProgramError::Custom(e as u32)
    }
}