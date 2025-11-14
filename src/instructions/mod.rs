use pinocchio::program_error::ProgramError;

pub mod claim;
pub mod ed25519_call;

pub use claim::*;
pub use ed25519_call::*;

#[repr(u8)]
pub enum ProgramInstruction {
    ClaimTokens,
    ED25519Call
}

impl TryFrom<&u8> for ProgramInstruction {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(ProgramInstruction::ED25519Call),
            1 => Ok(ProgramInstruction::ClaimTokens),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}