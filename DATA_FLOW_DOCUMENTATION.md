# Data Flow Documentation - Tornado Cash Solana

## Overview
This document describes the complete data flow from JavaScript proof generation through Solana on-chain verification.

## Data Flow Diagram

```
┌─────────────┐      ┌──────────────┐      ┌───────────────┐      ┌──────────────┐
│   Circom    │ ---> │   snarkjs    │ ---> │  JavaScript   │ ---> │    Rust      │
│   Circuit   │      │  Proof Gen   │      │  Formatting   │      │ Verification │
└─────────────┘      └──────────────┘      └───────────────┘      └──────────────┘
      ↓                     ↓                      ↓                      ↓
 8 Public Signals    Field Elements         256-byte Proof         Groth16 Verify
                     (Decimal Strings)       (Big-Endian)          (Native Syscalls)
```

## 1. Circuit Output (Circom)

The withdrawal circuit outputs exactly 8 public signals:

```javascript
// Public signals from circuit
[
  root,           // Merkle tree root
  nullifierHash,  // Nullifier to prevent double-spending
  recipientHigh,  // High 16 bytes of recipient address (padded)
  recipientLow,   // Low 16 bytes of recipient address (padded)
  relayerHigh,    // High 16 bytes of relayer address (padded)
  relayerLow,     // Low 16 bytes of relayer address (padded)
  fee,            // Transaction fee in lamports
  refund          // Refund amount in lamports
]
```

### Address Splitting in Circuit
Solana addresses (32 bytes) exceed BN254 field size (~31 bytes), so they're split:
- `addressHigh = [0; 16] || address[0:16]`
- `addressLow = [0; 16] || address[16:32]`

## 2. Proof Generation (snarkjs)

snarkjs outputs:
- **proof**: Object with `pi_a`, `pi_b`, `pi_c` (field elements as decimal strings)
- **publicSignals**: Array of 8 field elements (decimal strings)

### Proof Structure
```javascript
proof = {
  pi_a: [x, y],           // G1 point (2 field elements)
  pi_b: [[x1, x2], [y1, y2]], // G2 point (4 field elements)  
  pi_c: [x, y]            // G1 point (2 field elements)
}
```

## 3. JavaScript Formatting

### Field Element to Bytes Conversion
```javascript
function fieldToBytes32(fieldElement) {
    const hex = BigInt(fieldElement).toString(16).padStart(64, '0');
    return Buffer.from(hex, 'hex');
}
```

### Proof Formatting (256 bytes total)
```javascript
function formatProofForSolana(proof) {
    return Buffer.concat([
        fieldToBytes32(proof.pi_a[0]),     // 32 bytes
        fieldToBytes32(proof.pi_a[1]),     // 32 bytes (64 total for A)
        fieldToBytes32(proof.pi_b[0][1]),  // Note: coordinates swapped
        fieldToBytes32(proof.pi_b[0][0]),  // for G2 compatibility
        fieldToBytes32(proof.pi_b[1][1]),  
        fieldToBytes32(proof.pi_b[1][0]),  // (128 total for B)
        fieldToBytes32(proof.pi_c[0]),     // 32 bytes
        fieldToBytes32(proof.pi_c[1])      // 32 bytes (64 total for C)
    ]);
}
```

**Important**: G2 coordinates are swapped `[x,y] → [y,x]` for compatibility.

## 4. Rust Processing

### Step 4.1: Parse Proof Components
```rust
let proof_a_bytes = &proof[0..64];    // Uncompressed G1 point
let proof_b_bytes = &proof[64..192];  // Uncompressed G2 point
let proof_c_bytes = &proof[192..256]; // Uncompressed G1 point
```

### Step 4.2: Proof A Negation (Required for Circom)
```rust
fn negate_proof_a(proof_a_bytes: &[u8]) -> Result<[u8; 64], &'static str> {
    // 1. Convert big-endian to little-endian (ark-bn254 expects LE)
    let le_bytes = change_endianness(proof_a_bytes);
    
    // 2. Deserialize as G1 point (uncompressed)
    let point = G1Affine::deserialize_uncompressed(&le_bytes[..]);
    
    // 3. Negate the point (circom requirement)
    let negated = -point;
    
    // 4. Serialize back to bytes
    let mut output = vec![0u8; 64];
    negated.serialize_uncompressed(&mut output[..]);
    
    // 5. Convert back to big-endian (groth16-solana expects BE)
    change_endianness(&output)
}
```

### Step 4.3: Prepare Public Inputs
```rust
fn prepare_public_inputs(...) -> [[u8; 32]; 8] {
    [
        root,                                    // As-is
        nullifier_hash,                          // As-is
        split_address_to_high_low(recipient).0, // High part
        split_address_to_high_low(recipient).1, // Low part
        split_address_to_high_low(relayer).0,   // High part
        split_address_to_high_low(relayer).1,   // Low part
        encode_u64_as_32_bytes(fee),           // Right-aligned BE
        encode_u64_as_32_bytes(refund)         // Right-aligned BE
    ]
}
```

### Step 4.4: Verification
```rust
let mut verifier = Groth16Verifier::new(
    &proof_a_negated,  // Negated proof A
    &proof_b_bytes,    // Original proof B
    &proof_c_bytes,    // Original proof C
    &public_inputs,    // [[u8; 32]; 8]
    &verifying_key     // From trusted setup
)?;

verifier.verify()?; // Uses Solana's alt_bn128 syscalls (<200k CU)
```

## 5. Endianness Summary

### JavaScript → Rust
- **Input**: Big-endian 32-byte field elements
- **Processing**: No conversion needed initially

### Rust Internal (ark-bn254)
- **Input**: Needs little-endian
- **Processing**: Convert BE→LE for deserialization
- **Output**: Produces little-endian

### Rust → groth16-solana
- **Input**: Expects big-endian
- **Processing**: Convert LE→BE after ark operations
- **Output**: Verification result

## 6. Critical Implementation Notes

### ✅ Correct Patterns

1. **Double Endianness Conversion is REQUIRED**
   - BE→LE for ark-bn254 processing
   - LE→BE for groth16-solana verification
   - This is NOT a bug, it's the compatibility layer

2. **Proof A Negation is MANDATORY**
   - Circom/snarkjs convention requires negation
   - Must happen before verification

3. **Address Splitting Must Match**
   - JS and Rust must split addresses identically
   - Padding in high bytes ([0; 16] prefix)

### ⚠️ Common Pitfalls

1. **Wrong Deserialization**
   - Use `deserialize_uncompressed` (64 bytes)
   - NOT `deserialize_compressed` (32 bytes)

2. **Missing Negation**
   - Proof will always fail without negating A

3. **Endianness Confusion**
   - The double conversion is intentional
   - Don't "optimize" it away

4. **Public Input Count**
   - Must be exactly 8
   - Order must match circuit output

## 7. Testing Strategy

### Unit Tests
- `test_endianness_conversion`: Verify reversibility
- `test_address_splitting`: Verify split/reconstruct
- `test_fee_encoding`: Verify right-alignment
- `test_prepare_public_inputs`: Verify all 8 inputs

### Integration Tests
- `test_full_verification_flow`: End-to-end simulation
- `test_verify_proof_error_handling`: Error cases

### Manual Testing
```bash
# Generate test proof
node scripts/generate_test_proof.js

# Test on-chain
anchor test
```

## 8. Security Considerations

1. **Proof Validation**
   - Always check proof length (256 bytes)
   - Validate all components before processing

2. **Public Input Validation**
   - Verify addresses are valid Pubkeys
   - Check fee/refund don't overflow

3. **Error Handling**
   - Never panic on invalid input
   - Return descriptive errors

4. **Constant-Time Operations**
   - Critical for privacy preservation
   - Handled by ark-bn254 and groth16-solana

## 9. Performance

- **Proof Verification**: <200k compute units
- **Memory Usage**: ~10KB for proof + inputs
- **Processing Time**: <10ms on modern hardware

## Conclusion

The data flow is complex but well-defined. Each transformation serves a specific purpose:
1. Field elements → bytes for serialization
2. Big-endian → little-endian for ark-bn254
3. Proof A negation for circom compatibility  
4. Little-endian → big-endian for groth16-solana

Following this flow ensures successful proof verification on Solana.