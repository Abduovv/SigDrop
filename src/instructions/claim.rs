use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
    sysvars::instructions::{Instructions as IxSysvar, INSTRUCTIONS_ID},
};
use solana_program::{ed25519_program::ID as ED25519_ID, pubkey::Pubkey as SolPubkey};
use crate::errors::MyProgramError;
use crate::states::utils::*;
use core::i16;

pub fn claim(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Expected account order: recipient, vault, instruction sysvar, system program
    let [recipient, vault, ix_sysvar_acc, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Ensure caller signed the transaction
    if !recipient.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Validate the instruction sysvar account
    if ix_sysvar_acc.key() != &INSTRUCTIONS_ID {
        return Err(ProgramError::InvalidAccountData);
    }

    // Load the instruction sysvar to inspect previous instructions
    let ix_sysvar = unsafe { IxSysvar::new_unchecked(ix_sysvar_acc.try_borrow_data()?) };

    // Get the index of the currently executing instruction
    let idx = ix_sysvar.load_current_index() as usize;
    if idx == 0 {
        // There must be at least one instruction before this one
        return Err(ProgramError::InvalidInstructionData);
    }

    // Load the previous instruction (expected to be the ed25519 verification)
    let ed_ix = ix_sysvar.load_instruction_at(idx - 1)?;

    // Check that the previous instruction was sent to the ed25519 program
    if ed_ix.get_program_id() != ED25519_ID.as_array() {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Basic metadata validation on ed25519 instruction accounts
    if ed_ix.get_account_meta_at(0).unwrap().key == Pubkey::default() {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Extract raw ed25519 instruction data
    let ed_data = ed_ix.get_instruction_data();

    // Verify ed25519 header and ensure the instruction format is valid
    if ed_data.len() <= HEADER_LEN || ed_data[0] != 1 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Helper to read little-endian u16 offsets from input `data`
    let read_u16 = |i| -> Result<u16, ProgramError> {
        let o = 2 + 2 * i as usize;
        Ok(u16::from_le_bytes(
            data.get(o..o + 2).ok_or(ProgramError::InvalidAccountData)?.try_into().unwrap(),
        ))
    };

    // Extract all offset and index parameters from the ed25519 instruction descriptor
    let signature_offset = read_u16(0)? as usize;
    let signature_ix_idx = read_u16(1)? as usize;
    let public_key_offset = read_u16(2)? as usize;
    let public_key_ix_idx = read_u16(3)? as usize;
    let message_offset = read_u16(4)? as usize;
    let message_size = read_u16(5)? as usize;
    let message_ix_idx = read_u16(6)? as usize;

    // Offsets must refer to the current instruction; no cross-instruction references allowed
    let this = i16::MAX as usize;
    if signature_ix_idx != this || public_key_ix_idx != this || message_ix_idx != this {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Offsets must not fall inside the ed25519 header
    if signature_offset < HEADER_LEN
        || public_key_offset < HEADER_LEN
        || message_offset < HEADER_LEN
    {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Bounds check for signature, public key, and message segments
    if ed_data.len() < signature_offset + SIG_LEN
        || ed_data.len() < public_key_offset + PUBKEY_LEN
        || ed_data.len() < message_offset + message_size
        || message_size != MSG_LEN
    {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Extract the distributor public key used to sign the message
    let distributor = SolPubkey::new_from_array(
        ed_data[public_key_offset..public_key_offset + 32].try_into().unwrap(),
    );

    // Distributor must match an expected hard-coded key (here: the default key)
    if distributor != Pubkey::default().into() {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Extract and inspect the signed message
    let msg = &ed_data[message_offset..message_offset + message_size];

    // First 32 bytes of message represent the intended recipient's public key
    let rec_pubkey = SolPubkey::new_from_array(msg[0..32].try_into().unwrap());

    // Ensure recipient matches the public key in the signed message
    if rec_pubkey.as_array() != recipient.key() {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Extract the airdrop amount encoded in the signed message
    let amount = u64::from_le_bytes(msg[32..40].try_into().unwrap());

    // Safe lamport transfers using unchecked lamport references
    let mut vault_lamports = unsafe { vault.borrow_mut_lamports_unchecked() };
    let mut rec_lamports = unsafe { recipient.borrow_mut_lamports_unchecked() };

    // Decrement vault, increment recipient
    *vault_lamports = vault_lamports.checked_sub(amount).ok_or(MyProgramError::InsufficientFunds)?;
    *rec_lamports = rec_lamports.checked_add(amount).ok_or(MyProgramError::InsufficientFunds)?;

    Ok(())
}
