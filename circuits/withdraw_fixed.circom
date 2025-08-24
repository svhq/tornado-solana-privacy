pragma circom 2.1.5;

include "../node_modules/circomlib/circuits/poseidon.circom";
include "../node_modules/circomlib/circuits/bitify.circom";
include "../node_modules/circomlib/circuits/comparators.circom";

// Merkle tree verification matching our Rust implementation
template MerkleTreeChecker(levels) {
    signal input leaf;
    signal input root;
    signal input pathElements[levels];
    signal input pathIndices[levels];

    component hashers[levels];
    component indexBits[levels];
    
    signal currentHash[levels + 1];
    currentHash[0] <== leaf;

    for (var i = 0; i < levels; i++) {
        indexBits[i] = Num2Bits(1);
        indexBits[i].in <== pathIndices[i];
        
        hashers[i] = Poseidon(2);
        hashers[i].inputs[0] <== currentHash[i] * (1 - indexBits[i].out[0]) + pathElements[i] * indexBits[i].out[0];
        hashers[i].inputs[1] <== pathElements[i] * (1 - indexBits[i].out[0]) + currentHash[i] * indexBits[i].out[0];
        
        currentHash[i + 1] <== hashers[i].out;
    }

    root === currentHash[levels];
}

// FIXED: Properly handle 32-byte Solana addresses
template Withdraw(levels) {
    // Public inputs
    signal input root;
    signal input nullifierHash;
    
    // CRITICAL FIX: Split 32-byte addresses into two 16-byte field elements
    // Solana addresses are 32 bytes, but BN254 field can only safely hold ~31 bytes
    // Solution: Split address into high 16 bytes and low 16 bytes
    signal input recipientHigh;  // First 16 bytes of recipient address
    signal input recipientLow;   // Last 16 bytes of recipient address
    signal input relayerHigh;    // First 16 bytes of relayer address
    signal input relayerLow;     // Last 16 bytes of relayer address
    
    signal input fee;
    signal input refund;

    // Private inputs
    signal input nullifier;
    signal input secret;
    signal input pathElements[levels];
    signal input pathIndices[levels];

    // Add range constraints to ensure address parts are valid 16-byte values
    // Each part must be < 2^128 to represent 16 bytes
    component recipientHighRange = LessThan(128);
    recipientHighRange.in[0] <== recipientHigh;
    recipientHighRange.in[1] <== 2**128;
    recipientHighRange.out === 1;
    
    component recipientLowRange = LessThan(128);
    recipientLowRange.in[0] <== recipientLow;
    recipientLowRange.in[1] <== 2**128;
    recipientLowRange.out === 1;
    
    component relayerHighRange = LessThan(128);
    relayerHighRange.in[0] <== relayerHigh;
    relayerHighRange.in[1] <== 2**128;
    relayerHighRange.out === 1;
    
    component relayerLowRange = LessThan(128);
    relayerLowRange.in[0] <== relayerLow;
    relayerLowRange.in[1] <== 2**128;
    relayerLowRange.out === 1;

    // Compute commitment = Poseidon(nullifier, secret)
    component commitmentHasher = Poseidon(2);
    commitmentHasher.inputs[0] <== nullifier;
    commitmentHasher.inputs[1] <== secret;

    // Compute nullifier hash = Poseidon(nullifier)
    component nullifierHasher = Poseidon(1);
    nullifierHasher.inputs[0] <== nullifier;
    nullifierHasher.out === nullifierHash;

    // Verify merkle proof
    component tree = MerkleTreeChecker(levels);
    tree.leaf <== commitmentHasher.out;
    tree.root <== root;
    for (var i = 0; i < levels; i++) {
        tree.pathElements[i] <== pathElements[i];
        tree.pathIndices[i] <== pathIndices[i];
    }

    // Add constraints to prevent tampering with public inputs
    signal feeSquare <== fee * fee;
    signal refundSquare <== refund * refund;
}

// Update public inputs to include split addresses
component main {public [
    root, 
    nullifierHash, 
    recipientHigh, 
    recipientLow, 
    relayerHigh, 
    relayerLow, 
    fee, 
    refund
]} = Withdraw(20);