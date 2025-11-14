# SigDrop — Solana Signature-Gated Airdrop Program

**SigDrop** is a Solana on-chain program that enables secure, signature-gated airdrops. The main purpose of this program is to allow users to "claim" tokens or rewards from a vault, but only if they provide a valid off-chain signature, proving their eligibility. This pattern is often used for airdrop campaigns, rewards distributions, or any scenario where a central distributor wants to cryptographically authorize claims using Ed25519 signatures, without maintaining large on-chain whitelists.

## How It Works

1. **Distributor Signs Claim**: The distributor creates an Ed25519 signature for each recipient, encoding the recipient's public key and airdrop amount.
2. **User Submits Claim**: The recipient submits a transaction with the signed message and required accounts.
3. **On-chain Verification**:
   - The program uses the Ed25519 verification instruction to validate the signature.
   - It parses the message to check if the public key and amount match the intended recipient and transfer.
   - If verified, the vault's balance is reduced and the recipient's account balance is increased accordingly.

## Project Structure

```
src/
├── entrypoint.rs        # Main entrypoint, dispatching 'ClaimTokens' and 'ED25519Call' instructions
├── instructions/
│   ├── claim.rs         # Logic for claim instruction — signature validation and transfer
│   └── ed25519_call.rs  # Helper for verifying Ed25519 signatures
├── states/
│   └── utils.rs         # Low-level utilities for account and instruction data parsing
├── errors.rs            # Custom errors for robust failure handling
└── lib.rs               # Library crate for no_std Solana program logic
tests/
└── tests.ts            # Unit/integration tests 
```

## Example Usage Flow

1. Distributor computes claim signature for a user.
2. Recipient sends a transaction to the program containing:
    - Their own account as signer.
    - Vault account for distributing.
    - Ed25519 verification sysvar.
    - The signed message and Ed25519 instruction in the transaction data.
3. Program verifies the signature and distributes the tokens to the rightful recipient.
