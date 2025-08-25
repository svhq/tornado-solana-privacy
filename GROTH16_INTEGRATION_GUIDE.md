# Groth16 Integration Guide for Tornado Cash Solana

## Complete Understanding of groth16-solana

After thorough analysis of Light Protocol's groth16-solana, here's the complete integration approach for our Tornado Cash implementation.

## Critical API Requirements

### 1. Proof Format (256 bytes total)
```rust
// From snarkjs/circom output:
// proof[0..64]   = Proof A (G1 point) - MUST BE NEGATED!
// proof[64..192] = Proof B (G2 point)
// proof[192..256] = Proof C (G1 point)
```

### 2. Correct Groth16Verifier Usage

**WRONG (what we had):**
```rust
// This is why compilation failed
match Groth16Verifier::new(
    proof_a.try_into().unwrap_or(&[0u8; 64]),
    proof_b.try_into().unwrap_or(&[0u8; 128]),
    proof_c.try_into().unwrap_or(&[0u8; 64]),
    &public_inputs,        // Vec<&[u8]> - WRONG TYPE
    verifying_key,         // &[u8] - WRONG TYPE
)
```

**CORRECT:**
```rust
use groth16_solana::groth16::{Groth16Verifier, Groth16Verifyingkey};

// Define the verifying key (from circuit)
const VERIFYING_KEY: Groth16Verifyingkey = Groth16Verifyingkey {
    nr_pubinputs: 4,  // root, nullifier_hash, recipient, fee
    vk_alpha_g1: [/* 64 bytes */],
    vk_beta_g2: [/* 128 bytes */],
    vk_gamme_g2: [/* 128 bytes */],  // Note: typo in original
    vk_delta_g2: [/* 128 bytes */],
    vk_ic: &[/* Array of [u8; 64] IC points */],
};

fn verify_proof(
    proof: Vec<u8>,
    root: [u8; 32],
    nullifier_hash: [u8; 32],
    recipient: [u8; 32],
    fee: [u8; 32],
) -> Result<bool, ProgramError> {
    // Step 1: Process proof A with NEGATION
    let proof_a_bytes = &proof[0..64];
    let proof_a_negated = negate_g1_point(proof_a_bytes)?;
    
    // Step 2: Extract proof B and C
    let proof_b: [u8; 128] = proof[64..192].try_into()
        .map_err(|_| ProgramError::InvalidArgument)?;
    let proof_c: [u8; 64] = proof[192..256].try_into()
        .map_err(|_| ProgramError::InvalidArgument)?;
    
    // Step 3: Prepare public inputs (exact array size)
    let public_inputs: [[u8; 32]; 4] = [
        root,
        nullifier_hash,
        recipient,
        fee,
    ];
    
    // Step 4: Create and run verifier
    let mut verifier = Groth16Verifier::<4>::new(
        &proof_a_negated,
        &proof_b,
        &proof_c,
        &public_inputs,
        &VERIFYING_KEY,
    ).map_err(|_| ProgramError::InvalidArgument)?;
    
    verifier.verify()
        .map(|_| true)
        .map_err(|_| ProgramError::InvalidArgument)
}
```

## Helper Functions Needed

### 1. Negate G1 Point
```rust
use ark_ec::AffineRepr;
use ark_bn254::{G1Affine, Fq};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

fn negate_g1_point(point_bytes: &[u8]) -> Result<[u8; 64], ProgramError> {
    // Convert to little-endian for ark processing
    let le_bytes = change_endianness(point_bytes);
    
    // Deserialize as G1 point
    let mut reader = &le_bytes[..];
    let point = G1Affine::deserialize_uncompressed(&mut reader)
        .map_err(|_| ProgramError::InvalidArgument)?;
    
    // Negate the point
    let negated = -point;
    
    // Serialize back
    let mut output = vec![0u8; 64];
    negated.serialize_uncompressed(&mut output[..])
        .map_err(|_| ProgramError::InvalidArgument)?;
    
    // Convert back to big-endian
    let be_bytes = change_endianness(&output);
    
    be_bytes.try_into()
        .map_err(|_| ProgramError::InvalidArgument)
}

fn change_endianness(bytes: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    for chunk in bytes.chunks(32) {
        for byte in chunk.iter().rev() {
            result.push(*byte);
        }
    }
    result
}
```

### 2. Convert Addresses to Field Elements
```rust
fn address_to_field_element(address: &Pubkey) -> [u8; 32] {
    // Solana addresses are already 32 bytes
    // But we need to ensure they're < field size
    let mut bytes = address.to_bytes();
    
    // Clear the most significant bit to ensure < field size
    bytes[31] &= 0x7F;
    
    bytes
}
```

## Circuit Public Inputs Structure

For our Tornado Cash circuit, the public inputs are:

```rust
// Circuit expects these public inputs (in order):
// 1. root - Merkle tree root
// 2. nullifierHash - Hash of nullifier
// 3. recipientHash - Hash of recipient address
// 4. fee - Relayer fee

const NR_PUBLIC_INPUTS: usize = 4;
```

## Complete Integration for lib.rs

```rust
use anchor_lang::prelude::*;
use groth16_solana::groth16::{Groth16Verifier, Groth16Verifyingkey};
use ark_bn254::G1Affine;
use ark_ec::AffineRepr;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

// Include the generated verifying key
include!("verifying_key.rs");

#[program]
pub mod tornado_solana {
    use super::*;

    pub fn withdraw(
        ctx: Context<Withdraw>,
        proof: Vec<u8>,
        root: [u8; 32],
        nullifier_hash: [u8; 32],
        recipient: Pubkey,
        fee: u64,
    ) -> Result<()> {
        let tornado_state = &mut ctx.accounts.tornado_state;
        
        // Verify the merkle root is known
        require!(
            is_known_root(&tornado_state.roots, tornado_state.current_root_index, &root),
            TornadoError::UnknownRoot
        );
        
        // Check nullifier hasn't been used
        require!(
            !tornado_state.nullifier_hashes.contains(&nullifier_hash),
            TornadoError::AlreadySpent
        );
        
        // Prepare proof components
        require!(proof.len() == 256, TornadoError::InvalidProof);
        
        // CRITICAL: Negate proof A
        let proof_a_negated = negate_g1_point(&proof[0..64])?;
        let proof_b: [u8; 128] = proof[64..192].try_into()
            .map_err(|_| TornadoError::InvalidProof)?;
        let proof_c: [u8; 64] = proof[192..256].try_into()
            .map_err(|_| TornadoError::InvalidProof)?;
        
        // Prepare public inputs
        let recipient_hash = hash_address(&recipient);
        let fee_bytes = fee_to_bytes(fee);
        
        let public_inputs: [[u8; 32]; 4] = [
            root,
            nullifier_hash,
            recipient_hash,
            fee_bytes,
        ];
        
        // Verify the proof
        let mut verifier = Groth16Verifier::<4>::new(
            &proof_a_negated,
            &proof_b,
            &proof_c,
            &public_inputs,
            &VERIFYING_KEY,
        ).map_err(|_| TornadoError::InvalidProof)?;
        
        verifier.verify()
            .map_err(|_| TornadoError::InvalidProof)?;
        
        // Mark nullifier as spent
        tornado_state.nullifier_hashes.push(nullifier_hash);
        
        // Transfer funds
        let withdraw_amount = tornado_state.denomination - fee;
        
        **tornado_state.to_account_info().try_borrow_mut_lamports()? -= withdraw_amount;
        **ctx.accounts.recipient.try_borrow_mut_lamports()? += withdraw_amount;
        
        emit!(WithdrawalEvent {
            nullifier_hash,
            recipient,
            fee,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }
}

// Helper functions
fn negate_g1_point(point_bytes: &[u8]) -> Result<[u8; 64], ProgramError> {
    // Implementation as shown above
}

fn hash_address(address: &Pubkey) -> [u8; 32] {
    // Use Poseidon to hash the address
    use crate::merkle_tree::MerkleTree;
    MerkleTree::hash_leaf(&address.to_bytes())
}

fn fee_to_bytes(fee: u64) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[24..].copy_from_slice(&fee.to_be_bytes());
    bytes
}
```

## Generating the Verifying Key

### Step 1: Generate from Circuit
```bash
# After circuit compilation and trusted setup
snarkjs zkey export verificationkey withdraw_final.zkey verification_key.json
```

### Step 2: Convert to Rust
```bash
cd circuits
npm install ffjavascript
node ../Light\ Protocol\ Github\ Repos/groth16-solana-master/parse_vk_to_rust.js verification_key.json ../programs/tornado_solana/src/
```

This generates `verifying_key.rs` with the correct format.

### Step 3: Include in Program
```rust
// In lib.rs
include!("verifying_key.rs");
// This provides: const VERIFYING_KEY: Groth16Verifyingkey
```

## Common Pitfalls Fixed

### 1. ✅ Proof A Negation
- We now properly negate proof A using ark-bn254

### 2. ✅ Type Mismatch
- Public inputs are now `[[u8; 32]; 4]` not `Vec<&[u8]>`
- Verifying key is now `Groth16Verifyingkey` not `&[u8]`

### 3. ✅ Const Generics
- Using `Groth16Verifier::<4>` with exact public input count

### 4. ✅ Endianness
- Properly converting between big-endian (Solana) and little-endian (ark)

## Dependencies Update

```toml
[dependencies]
anchor-lang = "0.29.0"
anchor-spl = "0.29.0"
solana-program = "1.18.17"
light-poseidon = "0.2.0"
ark-bn254 = "0.4.0"
groth16-solana = "0.2.0"
ark-serialize = "0.4"
ark-ff = "0.4"
ark-ec = "0.4"

[dev-dependencies]
hex = "0.4"
```

## Testing the Integration

### 1. Generate Test Proof
```javascript
// In circuits/test_proof.js
const snarkjs = require("snarkjs");

async function generateTestProof() {
    const input = {
        // Public inputs
        root: "0x1234...",
        nullifierHash: "0x5678...",
        recipient: "0xabcd...",
        fee: "1000000",
        
        // Private inputs
        nullifier: "123456",
        secret: "789012",
        pathElements: [...],
        pathIndices: [...]
    };
    
    const { proof, publicSignals } = await snarkjs.groth16.fullProve(
        input,
        "withdraw.wasm",
        "withdraw_final.zkey"
    );
    
    // Convert proof to bytes
    const proofBytes = [
        ...hexToBytes(proof.pi_a[0]),
        ...hexToBytes(proof.pi_a[1]),
        ...hexToBytes(proof.pi_b[0][1]),
        ...hexToBytes(proof.pi_b[0][0]),
        ...hexToBytes(proof.pi_b[1][1]),
        ...hexToBytes(proof.pi_b[1][0]),
        ...hexToBytes(proof.pi_c[0]),
        ...hexToBytes(proof.pi_c[1])
    ];
    
    console.log("Proof bytes:", proofBytes);
    console.log("Public signals:", publicSignals);
}
```

### 2. Test in Solana
```rust
#[test]
fn test_proof_verification() {
    let proof = vec![/* 256 bytes from test */];
    let root = [/* 32 bytes */];
    let nullifier_hash = [/* 32 bytes */];
    let recipient = Pubkey::new_unique();
    let fee = 1000000u64;
    
    let result = verify_proof(proof, root, nullifier_hash, recipient, fee);
    assert!(result.is_ok());
}
```

## Summary

With this complete understanding of groth16-solana:

1. ✅ We know exactly how to format our data
2. ✅ We understand the proof A negation requirement
3. ✅ We have the correct API usage
4. ✅ We know how to generate and convert verifying keys
5. ✅ We can properly handle endianness conversions

The system is now ready to be properly integrated with Groth16 verification!