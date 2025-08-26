# Verifying Key Setup Guide

## Overview
This guide explains how to generate and integrate the verifying key for the Tornado Cash Solana implementation.

## Prerequisites

- Circom circuit compiled (`withdraw.circom`)
- snarkjs installed (`npm install -g snarkjs`)
- Powers of Tau ceremony file

## Step 1: Circuit Compilation

```bash
# Compile the circuit
circom withdraw.circom --r1cs --wasm --sym

# Generate circuit info
snarkjs r1cs info withdraw.r1cs
```

Expected output should show 8 public inputs:
- root
- nullifierHash
- recipientHigh
- recipientLow
- relayerHigh
- relayerLow
- fee
- refund

## Step 2: Trusted Setup

### Phase 1: Powers of Tau (Can reuse existing)

```bash
# Download existing Powers of Tau
wget https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_20.ptau
```

### Phase 2: Circuit-Specific Setup

```bash
# Generate initial zkey
snarkjs groth16 setup withdraw.r1cs powersOfTau28_hez_final_20.ptau withdraw_0000.zkey

# Contribute randomness (repeat with multiple contributors)
snarkjs zkey contribute withdraw_0000.zkey withdraw_0001.zkey --name="Contributor 1" -v

# Verify the contribution
snarkjs zkey verify withdraw.r1cs powersOfTau28_hez_final_20.ptau withdraw_0001.zkey

# Apply random beacon (final step)
snarkjs zkey beacon withdraw_0001.zkey withdraw_final.zkey 0x[random_hex] 10 -n="Final Beacon"

# Export verification key
snarkjs zkey export verificationkey withdraw_final.zkey verification_key.json
```

## Step 3: Convert to Rust Format

```bash
# Use our conversion script
node scripts/parse_vk_to_rust.js verification_key.json > programs/tornado_solana/src/verifying_key.rs
```

## Step 4: Integration

Replace the placeholder in `lib.rs`:

```rust
// Import the generated verifying key
mod verifying_key;
use verifying_key::WITHDRAWAL_VERIFYING_KEY;

// In verify_proof function, replace:
// &PLACEHOLDER_VERIFYING_KEY
// with:
// &WITHDRAWAL_VERIFYING_KEY
```

## Step 5: Testing

### Generate Test Proof

```javascript
const { proof, publicSignals } = await snarkjs.groth16.fullProve(
    {
        // Private inputs
        root: rootHex,
        nullifierHash: nullifierHashHex,
        recipient: recipientBytes,
        relayer: relayerBytes,
        fee: feeAmount,
        refund: refundAmount,
        // ... other private inputs
    },
    "withdraw.wasm",
    "withdraw_final.zkey"
);

// Format proof for Solana (256 bytes)
const solidityProof = await snarkjs.groth16.exportSolidityCallData(proof, publicSignals);
```

### Verify On-Chain

The Solana program will:
1. Parse the 256-byte proof into A, B, C components
2. Negate proof A (for circom compatibility)
3. Prepare 8 public inputs
4. Call groth16-solana verifier

## Security Considerations

1. **Trusted Setup Security**:
   - Use multiple contributors (minimum 5-7)
   - At least one honest contributor ensures security
   - Document all contributors publicly

2. **Verifying Key Storage**:
   - Store hash on-chain for verification
   - Allow governed updates with timelock

3. **Circuit Constraints**:
   - Verify all 8 public inputs match circuit
   - Test with known valid/invalid proofs

## Troubleshooting

### Issue: Proof verification fails
- Check proof A is properly negated
- Verify public inputs order matches circuit
- Ensure endianness conversions are correct

### Issue: Wrong number of public inputs
- Circuit must output exactly 8 public signals
- IC array should have 9 points (8 + 1)

### Issue: Serialization errors
- snarkjs outputs uncompressed points (64 bytes for G1)
- Ensure using `deserialize_uncompressed`

## Production Checklist

- [ ] Trusted setup with 5+ contributors
- [ ] Verification key hash stored on-chain
- [ ] Test proofs verified successfully
- [ ] Circuit constraints documented
- [ ] Emergency pause mechanism ready
- [ ] Upgrade path defined (if any)