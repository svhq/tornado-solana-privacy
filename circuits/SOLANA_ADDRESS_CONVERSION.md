# Solana Address Conversion Strategy

## Problem Statement

**Critical Issue**: Solana addresses are 32 bytes (256 bits) but BN254 field elements can only safely hold ~254 bits (~31.75 bytes). Directly converting a Solana address to a field element causes overflow and data loss.

## Solution: Split Address Representation

Split each 32-byte Solana address into two 16-byte field elements:
- **High Part**: First 16 bytes (bytes 0-15)
- **Low Part**: Last 16 bytes (bytes 16-31)

This ensures each part fits safely within the field (16 bytes = 128 bits << 254 bits).

## Implementation

### 1. JavaScript/TypeScript (Client Side)

```javascript
// Convert Solana address to circuit inputs
function addressToCircuitInputs(address) {
    // address is either a PublicKey object or base58 string
    const bytes = typeof address === 'string' 
        ? bs58.decode(address) 
        : address.toBytes();
    
    // Split into two 16-byte parts
    const high = bytes.slice(0, 16);
    const low = bytes.slice(16, 32);
    
    // Convert to BigInt field elements
    const highField = BigInt('0x' + Buffer.from(high).toString('hex'));
    const lowField = BigInt('0x' + Buffer.from(low).toString('hex'));
    
    return {
        high: highField.toString(),
        low: lowField.toString()
    };
}

// Example usage for withdrawal
async function prepareWithdrawal(recipient, relayer) {
    const recipientParts = addressToCircuitInputs(recipient);
    const relayerParts = addressToCircuitInputs(relayer);
    
    const input = {
        // Public inputs
        root: merkleRoot,
        nullifierHash: computedNullifierHash,
        recipientHigh: recipientParts.high,
        recipientLow: recipientParts.low,
        relayerHigh: relayerParts.high,
        relayerLow: relayerParts.low,
        fee: feeAmount,
        refund: refundAmount,
        
        // Private inputs
        nullifier: nullifier,
        secret: secret,
        pathElements: merkleProof.pathElements,
        pathIndices: merkleProof.pathIndices
    };
    
    // Generate proof
    const { proof, publicSignals } = await snarkjs.groth16.fullProve(
        input,
        "circuits/build/withdraw_js/withdraw.wasm",
        "circuits/build/withdraw_final.zkey"
    );
    
    return { proof, publicSignals };
}
```

### 2. Rust/Solana (On-Chain)

```rust
// In lib.rs - reconstruct addresses from field elements
pub fn withdraw(
    ctx: Context<Withdraw>,
    proof: Vec<u8>,
    root: [u8; 32],
    nullifier_hash: [u8; 32],
    recipient_high: [u8; 16],  // First 16 bytes
    recipient_low: [u8; 16],    // Last 16 bytes
    relayer_high: [u8; 16],     // First 16 bytes
    relayer_low: [u8; 16],      // Last 16 bytes
    fee: u64,
    refund: u64,
) -> Result<()> {
    // Reconstruct full 32-byte addresses
    let mut recipient_bytes = [0u8; 32];
    recipient_bytes[..16].copy_from_slice(&recipient_high);
    recipient_bytes[16..].copy_from_slice(&recipient_low);
    let recipient = Pubkey::from(recipient_bytes);
    
    let mut relayer_bytes = [0u8; 32];
    relayer_bytes[..16].copy_from_slice(&relayer_high);
    relayer_bytes[16..].copy_from_slice(&relayer_low);
    let relayer = Pubkey::from(relayer_bytes);
    
    // Continue with withdrawal logic...
}
```

### 3. Circuit (Circom)

```circom
// In withdraw_fixed.circom
template Withdraw(levels) {
    // Split addresses into high/low parts
    signal input recipientHigh;  // First 16 bytes
    signal input recipientLow;   // Last 16 bytes
    signal input relayerHigh;    // First 16 bytes
    signal input relayerLow;     // Last 16 bytes
    
    // Add range constraints to ensure valid 16-byte values
    component recipientHighRange = LessThan(128);
    recipientHighRange.in[0] <== recipientHigh;
    recipientHighRange.in[1] <== 2**128;
    recipientHighRange.out === 1;
    
    // ... similar for other address parts
}
```

## Security Considerations

### 1. Range Validation
Each 16-byte part MUST be validated to be < 2^128 to prevent overflow attacks.

### 2. Reconstruction Order
Always maintain consistent byte ordering:
- High part = bytes[0:16]
- Low part = bytes[16:32]

### 3. Field Safety
16 bytes (128 bits) is safely within BN254 field size (~254 bits), providing ~126 bits of safety margin.

## Testing

### Test Vector
```javascript
// Test address (base58)
const testAddress = "11111111111111111111111111111112";

// Expected values
const expectedBytes = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  // High
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1   // Low
];

// Circuit inputs
const high = "0";  // 0x00000000000000000000000000000000
const low = "1";   // 0x00000000000000000000000000000001
```

## Migration Guide

### For Existing Code

**Before (BROKEN):**
```javascript
const input = {
    recipient: recipientAddress,  // Won't work - 32 bytes don't fit
    relayer: relayerAddress,      // Won't work
    ...
};
```

**After (FIXED):**
```javascript
const recipientParts = addressToCircuitInputs(recipientAddress);
const relayerParts = addressToCircuitInputs(relayerAddress);

const input = {
    recipientHigh: recipientParts.high,
    recipientLow: recipientParts.low,
    relayerHigh: relayerParts.high,
    relayerLow: relayerParts.low,
    ...
};
```

## Verification

To verify correct implementation:

1. Generate proof with split addresses
2. Verify proof passes circuit constraints
3. Reconstruct addresses on-chain
4. Confirm withdrawal to correct recipient

## Summary

This solution:
- ✅ Handles full 32-byte Solana addresses
- ✅ Maintains mathematical safety in BN254 field
- ✅ Preserves all address bits without loss
- ✅ Works with existing groth16-solana verifier
- ✅ Adds minimal overhead (~512 constraints)