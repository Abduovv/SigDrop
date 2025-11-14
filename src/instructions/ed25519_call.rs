use pinocchio::{
    account_info::AccountInfo,
    ProgramResult,
};
use solana_program::program::invoke;
use crate::states::{utils::load_ix_data, DataLen};
use solana_ed25519_program::new_ed25519_instruction_with_signature;


pub struct Ed25519CallData {
    pub message: [u8; 40],
    pub pubkey: [u8; 32],
    pub signature: [u8; 64],
}

impl DataLen for Ed25519CallData {
    const LEN: usize = 136;
}

pub fn ed25519_call(_accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Unsafe load of instruction data into typed struct
    let data_state = unsafe { load_ix_data::<Ed25519CallData>(data)? };

    // Create the ed25519 instruction
    let ed25519_instruction = new_ed25519_instruction_with_signature(
        &data_state.message,
        &data_state.signature,
        &data_state.pubkey,
    );

    // Invoke the ed25519 program CPI
   invoke(&ed25519_instruction, &[]).map_err(|_| pinocchio::program_error::ProgramError::Custom(0))?;

    Ok(())
}
