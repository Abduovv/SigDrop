import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  SystemProgram,
  TransactionInstruction,
  LAMPORTS_PER_SOL,
  Ed25519Program,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import nacl from 'tweetnacl';
import BN from 'bn.js';

// Async test function
(async () => {
  const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
  // Generate test accounts
  const vault = Keypair.generate();
  const distributor = Keypair.generate();
  const recipient = Keypair.generate();

  // Airdrop funds to vault
  await connection.requestAirdrop(vault.publicKey, LAMPORTS_PER_SOL);
  await new Promise(res => setTimeout(res, 5000)); // Wait for airdrop

  // Prepare claim message: [recipient pubkey | amount]
  const amount = new BN(LAMPORTS_PER_SOL / 10);
  const recipientPubkeyBytes = recipient.publicKey.toBytes();
  const amountBytes = Buffer.alloc(8);
  amount.toArrayLike(Buffer, 'le').copy(amountBytes);
  let message = Buffer.concat([recipientPubkeyBytes, amountBytes]);
  // Pad to 40 bytes
  if (message.length < 40) message = Buffer.concat([message, Buffer.alloc(40 - message.length)]);

  // Sign message off-chain
  const signature = nacl.sign.detached(message, distributor.secretKey);

  // Ed25519 verification instruction (placed first in tx)
  const ed25519Instruction = Ed25519Program.createInstructionWithPublicKey({
    publicKey: distributor.publicKey.toBytes(),
    message,
    signature,
    instructionIndex: 0
  });

  // YOU MUST encode claim data correctly for your program
  // This is a placeholder. You need to follow your Rust instruction format offsets and indices.
  // For illustrative purposes, here is a dummy 16-byte buffer:
  // Replace with exact data encoding expected by your contract.
  const claimIxData = Buffer.alloc(16);

  // Claim instruction for your SigDrop program
  const SIGDROP_PROGRAM_ID = new PublicKey('8zASAJ7QL5t7S2oSTyAanSFFehAB2i3n4LiRHsZ6piuZ'); // Replace with your actual program ID if different
  const claimInstruction = new TransactionInstruction({
    programId: SIGDROP_PROGRAM_ID,
    keys: [
      { pubkey: recipient.publicKey, isSigner: true, isWritable: true },
      { pubkey: vault.publicKey, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_INSTRUCTIONS_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: claimIxData, // You must provide the proper offsets/instruction data
  });

  // Compose transaction: ed25519 verification first, then the claim
  const tx = new Transaction().add(ed25519Instruction, claimInstruction);
  tx.feePayer = recipient.publicKey;

  // Send the transaction
  await sendAndConfirmTransaction(
    connection,
    tx,
    [recipient], // required signer
  );

  // Check result
  const vaultBalance = await connection.getBalance(vault.publicKey);
  const recipientBalance = await connection.getBalance(recipient.publicKey);
  console.log('Vault:', vaultBalance, 'Recipient:', recipientBalance);
})();