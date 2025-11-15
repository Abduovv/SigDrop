use pinocchio::{
    account_info::AccountInfo,
    instruction::Instruction,
    program::invoke,
    ProgramResult,
};
use solana_ed25519_program::new_ed25519_instruction_with_signature;
use crate::states::{utils::load_ix_data, DataLen};


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
    let ix_instruction = Instruction {
        program_id: ed25519_instruction.program_id.as_array(),
        accounts: &[].to_vec(),
        data: &ed25519_instruction.data,
    };

    invoke(&ix_instruction, &[])?;
    Ok(())
}

