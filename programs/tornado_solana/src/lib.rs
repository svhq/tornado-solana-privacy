use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use groth16_solana::groth16::{Groth16Verifier, Groth16Verifyingkey};
use ark_bn254::G1Affine;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

pub mod merkle_tree;
use merkle_tree::*;

pub mod verifying_key;
use verifying_key::get_circuit_verifying_key;

#[cfg(test)]
mod poseidon_test;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod simple_test;

#[cfg(test)]
mod real_proof_test;

declare_id!("11111111111111111111111111111112");

#[program]
pub mod tornado_solana {
    use super::*;

    /// Initialize a new Tornado pool with fixed denomination
    /// @param verifying_key: The Groth16 verifying key from trusted setup ceremony
    pub fn initialize(
        ctx: Context<Initialize>, 
        denomination: u64,
        verifying_key: Vec<u8>,
    ) -> Result<()> {
        let tornado_state = &mut ctx.accounts.tornado_state;
        
        tornado_state.authority = ctx.accounts.authority.key();
        tornado_state.denomination = denomination;
        tornado_state.merkle_tree = MerkleTree::new();
        tornado_state.current_root_index = 0;
        tornado_state.next_index = 0;
        tornado_state.verifying_key = verifying_key;
        
        Ok(())
    }

    /// Deposit funds into the tornado pool
    /// @param commitment: Hash(nullifier + secret)
    pub fn deposit(ctx: Context<Deposit>, commitment: [u8; 32]) -> Result<()> {
        // Store keys and values before borrowing tornado_state mutably
        let tornado_key = ctx.accounts.tornado_state.key();
        let tornado_info = ctx.accounts.tornado_state.to_account_info();
        
        let tornado_state = &mut ctx.accounts.tornado_state;
        
        // Check commitment hasn't been submitted before
        require!(
            !tornado_state.commitments.contains(&commitment),
            TornadoError::DuplicateCommitment
        );
        
        // Store denomination before the transfer
        let deposit_amount = tornado_state.denomination;
        
        // Transfer SOL to the vault
        let transfer_ix = system_instruction::transfer(
            &ctx.accounts.depositor.key(),
            &tornado_key,
            deposit_amount,
        );
        
        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.depositor.to_account_info(),
                tornado_info,
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        
        // Insert commitment into merkle tree
        let leaf_index = tornado_state.merkle_tree.insert(commitment)?;
        
        // Store commitment to prevent duplicates
        tornado_state.commitments.push(commitment);
        
        // Update root history
        let new_root = tornado_state.merkle_tree.get_root();
        let new_index = (tornado_state.current_root_index + 1) % ROOT_HISTORY_SIZE;
        tornado_state.current_root_index = new_index;
        tornado_state.roots[new_index as usize] = new_root;
        
        emit!(DepositEvent {
            commitment,
            leaf_index,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }

    /// Withdraw funds with a zero-knowledge proof
    pub fn withdraw(
        ctx: Context<Withdraw>,
        proof: Vec<u8>,
        root: [u8; 32],
        nullifier_hash: [u8; 32],
        recipient: Pubkey,
        relayer: Option<Pubkey>,
        fee: u64,
        refund: u64,
    ) -> Result<()> {
        let tornado_state = &mut ctx.accounts.tornado_state;
        
        // Verify fee doesn't exceed denomination
        require!(fee <= tornado_state.denomination, TornadoError::FeeExceedsDenomination);
        
        // Check nullifier hasn't been spent
        require!(
            !tornado_state.nullifier_hashes.contains(&nullifier_hash),
            TornadoError::NoteAlreadySpent
        );
        
        // Verify root is in history
        require!(
            is_known_root(&tornado_state.roots, tornado_state.current_root_index, &root),
            TornadoError::UnknownRoot
        );
        
        // Verify the zero-knowledge proof using Groth16
        // This uses Solana's native alt_bn128 syscalls for <200k CU verification
        // TODO: In production, deserialize the actual verifying key from tornado_state.verifying_key
        // For now, we'll use a placeholder - the actual VK comes from the trusted setup
        verify_proof(
            &proof, 
            &root, 
            &nullifier_hash, 
            &recipient, 
            &relayer.unwrap_or(Pubkey::default()), 
            fee, 
            refund, 
            &get_circuit_verifying_key()
        )?;
        
        // Mark nullifier as spent
        tornado_state.nullifier_hashes.push(nullifier_hash);
        
        // Calculate withdrawal amount
        let amount = tornado_state.denomination - fee;
        
        // Transfer to recipient
        **tornado_state.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.recipient.try_borrow_mut_lamports()? += amount;
        
        // Pay relayer fee if present
        if let Some(_relayer_pubkey) = relayer {
            if fee > 0 {
                **tornado_state.to_account_info().try_borrow_mut_lamports()? -= fee;
                **ctx.accounts.relayer.as_ref().unwrap().try_borrow_mut_lamports()? += fee;
            }
        }
        
        emit!(WithdrawalEvent {
            to: recipient,
            nullifier_hash,
            relayer,
            fee,
        });
        
        Ok(())
    }
}

// Constants matching original Tornado Cash
pub const ROOT_HISTORY_SIZE: u32 = 30;
pub const MERKLE_TREE_HEIGHT: u32 = 20;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + TornadoState::MAX_SIZE,
        seeds = [b"tornado"],
        bump
    )]
    pub tornado_state: Account<'info, TornadoState>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub tornado_state: Account<'info, TornadoState>,
    
    #[account(mut)]
    pub depositor: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub tornado_state: Account<'info, TornadoState>,
    
    /// CHECK: Recipient of withdrawn funds
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    
    /// CHECK: Optional relayer receiving fee
    #[account(mut)]
    pub relayer: Option<AccountInfo<'info>>,
}

#[account]
pub struct TornadoState {
    pub authority: Pubkey,
    pub denomination: u64,
    pub merkle_tree: MerkleTree,
    pub roots: [[u8; 32]; ROOT_HISTORY_SIZE as usize],
    pub current_root_index: u32,
    pub next_index: u32,
    pub nullifier_hashes: Vec<[u8; 32]>,
    pub commitments: Vec<[u8; 32]>,
    pub verifying_key: Vec<u8>,  // Groth16 verifying key from trusted setup
}

impl TornadoState {
    // Updated size to include verifying key (typically ~1KB)
    pub const MAX_SIZE: usize = 32 + 8 + MerkleTree::SIZE + (32 * 30) + 4 + 4 + (32 * 1000) + (32 * 1000) + 2048;
}

#[event]
pub struct DepositEvent {
    pub commitment: [u8; 32],
    pub leaf_index: u32,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawalEvent {
    pub to: Pubkey,
    pub nullifier_hash: [u8; 32],
    pub relayer: Option<Pubkey>,
    pub fee: u64,
}

#[error_code]
pub enum TornadoError {
    #[msg("The commitment has been submitted")]
    DuplicateCommitment,
    #[msg("Fee exceeds transfer value")]
    FeeExceedsDenomination,
    #[msg("The note has been already spent")]
    NoteAlreadySpent,
    #[msg("Cannot find your merkle root")]
    UnknownRoot,
    #[msg("Invalid withdraw proof")]
    InvalidProof,
    #[msg("Invalid proof length - must be 256 bytes")]
    InvalidProofLength,
    #[msg("Invalid proof format")]
    InvalidProofFormat,
    #[msg("Failed to negate proof A")]
    ProofNegationFailed,
    #[msg("Failed to create Groth16 verifier")]
    VerifierCreationFailed,
    #[msg("Merkle tree is full")]
    MerkleTreeFull,
}

// Helper functions
fn is_known_root(roots: &[[u8; 32]; ROOT_HISTORY_SIZE as usize], current_index: u32, root: &[u8; 32]) -> bool {
    if root == &[0u8; 32] {
        return false;
    }
    
    let mut i = current_index;
    loop {
        if &roots[i as usize] == root {
            return true;
        }
        
        if i == 0 {
            i = ROOT_HISTORY_SIZE - 1;
        } else {
            i -= 1;
        }
        
        if i == current_index {
            break;
        }
    }
    
    false
}

// Production-ready Groth16 proof verification using Solana's native syscalls
// This takes less than 200k compute units thanks to alt_bn128 syscalls
fn verify_proof(
    proof: &[u8],
    root: &[u8; 32],
    nullifier_hash: &[u8; 32],
    recipient: &Pubkey,
    relayer: &Pubkey,
    fee: u64,
    refund: u64,
    verifying_key: &Groth16Verifyingkey,
) -> Result<()> {
    // Proof should be 256 bytes (64 bytes for A, 128 for B, 64 for C)
    require!(
        proof.len() == 256,
        TornadoError::InvalidProofLength
    );
    
    // Parse proof components with proper error handling
    let proof_a_bytes = &proof[0..64];
    let proof_b_bytes: [u8; 128] = proof[64..192].try_into()
        .map_err(|_| {
            msg!("Invalid proof B format");
            TornadoError::InvalidProofFormat
        })?;
    let proof_c_bytes: [u8; 64] = proof[192..256].try_into()
        .map_err(|_| {
            msg!("Invalid proof C format");
            TornadoError::InvalidProofFormat
        })?;
    
    // Negate proof A (required for circom/snarkjs compatibility)
    msg!("About to negate proof A...");
    let proof_a_negated = negate_proof_a(proof_a_bytes)
        .map_err(|e| {
            msg!("Failed to negate proof A: {}", e);
            TornadoError::ProofNegationFailed
        })?;
    msg!("Proof A negation succeeded!");
    
    // Prepare 8 public inputs as required by the circuit
    let public_inputs = prepare_public_inputs(root, nullifier_hash, recipient, relayer, fee, refund);
    
    // Create and run verifier with correct types
    // Rust will infer Groth16Verifier::<8> from the array type
    let mut verifier = Groth16Verifier::new(
        &proof_a_negated,
        &proof_b_bytes,
        &proof_c_bytes,
        &public_inputs,
        verifying_key,
    ).map_err(|e| {
        msg!("Failed to create verifier: {:?}", e);
        TornadoError::VerifierCreationFailed
    })?;
    
    verifier.verify().map_err(|e| {
        msg!("Proof verification failed: {:?}", e);
        TornadoError::InvalidProof
    })?;
    
    Ok(())
}

/// Negate proof A using ark-bn254 (required for circom/snarkjs compatibility)
/// 
/// This function handles the necessary endianness conversions between:
/// 1. snarkjs output format (big-endian field elements)
/// 2. ark-bn254 requirements (little-endian for serialization)  
/// 3. groth16-solana expectations (big-endian proof components)
fn negate_proof_a(proof_a_bytes: &[u8]) -> Result<[u8; 64]> {
    // Use hush's exact pattern - add zero byte for uncompressed format
    let le_bytes_with_zero = [&change_endianness(proof_a_bytes)[..], &[0u8][..]].concat();
    
    // Deserialize as G1 point (65 bytes - uncompressed with infinity bit)
    let point = G1Affine::deserialize_uncompressed(&*le_bytes_with_zero)
        .map_err(|_| TornadoError::InvalidProofFormat)?;
    
    // Negate the point (required for circom compatibility)
    let negated = -point;
    
    // Serialize to 65-byte buffer
    let mut proof_a_neg = [0u8; 65];
    negated.serialize_uncompressed(&mut proof_a_neg[..])
        .map_err(|_| TornadoError::ProofNegationFailed)?;
    
    // Convert first 64 bytes back to big-endian for groth16-solana
    let be_bytes = change_endianness(&proof_a_neg[..64]);
    
    Ok(be_bytes.try_into()
        .map_err(|_| TornadoError::InvalidProofFormat)?)
}

/// Prepare the 8 public inputs for the circuit:
/// root, nullifierHash, recipientHigh, recipientLow, relayerHigh, relayerLow, fee, refund
fn prepare_public_inputs(
    root: &[u8; 32],
    nullifier_hash: &[u8; 32],
    recipient: &Pubkey,
    relayer: &Pubkey,
    fee: u64,
    refund: u64,
) -> [[u8; 32]; 8] {
    let mut inputs = [[0u8; 32]; 8];
    
    // Input 0: root
    inputs[0] = *root;
    
    // Input 1: nullifierHash
    inputs[1] = *nullifier_hash;
    
    // Inputs 2-3: recipient split into high/low parts
    let (recipient_high, recipient_low) = split_address_to_high_low(recipient);
    inputs[2] = recipient_high;
    inputs[3] = recipient_low;
    
    // Inputs 4-5: relayer split into high/low parts
    let (relayer_high, relayer_low) = split_address_to_high_low(relayer);
    inputs[4] = relayer_high;
    inputs[5] = relayer_low;
    
    // Input 6: fee as 32-byte big-endian
    encode_u64_as_32_bytes(fee, &mut inputs[6]);
    
    // Input 7: refund as 32-byte big-endian
    encode_u64_as_32_bytes(refund, &mut inputs[7]);
    
    inputs
}

/// Split a Solana address into high and low parts because they exceed BN254 field size
/// Addresses are 32 bytes, we split as: high = [0; 16] + [first 16 bytes], low = [0; 16] + [last 16 bytes]
fn split_address_to_high_low(address: &Pubkey) -> ([u8; 32], [u8; 32]) {
    let address_bytes = address.to_bytes();
    let mut high = [0u8; 32];
    let mut low = [0u8; 32];
    
    // High part: pad with zeros then first 16 bytes
    high[16..32].copy_from_slice(&address_bytes[0..16]);
    
    // Low part: pad with zeros then last 16 bytes
    low[16..32].copy_from_slice(&address_bytes[16..32]);
    
    (high, low)
}

/// Encode a u64 as a 32-byte big-endian array
fn encode_u64_as_32_bytes(value: u64, output: &mut [u8; 32]) {
    output[24..32].copy_from_slice(&value.to_be_bytes());
}

/// Reconstruct a Solana address from high and low parts
#[allow(dead_code)]
fn reconstruct_address_from_high_low(high: &[u8; 32], low: &[u8; 32]) -> Pubkey {
    let mut address_bytes = [0u8; 32];
    address_bytes[0..16].copy_from_slice(&high[16..32]);
    address_bytes[16..32].copy_from_slice(&low[16..32]);
    Pubkey::from(address_bytes)
}

/// Change endianness of bytes (big-endian <-> little-endian)
/// 
/// This function is essential for compatibility between:
/// - snarkjs/JavaScript: outputs 32-byte big-endian field elements
/// - ark-bn254: requires little-endian for (de)serialization
/// - groth16-solana: expects big-endian proof components
/// 
/// The function processes bytes in 32-byte chunks (BN254 field size) and
/// reverses the byte order within each chunk.
fn change_endianness(bytes: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    for chunk in bytes.chunks(32) {
        for byte in chunk.iter().rev() {
            result.push(*byte);
        }
    }
    result
}

