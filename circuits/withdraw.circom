pragma circom 2.1.5;

include "../node_modules/circomlib/circuits/poseidon.circom";
include "../node_modules/circomlib/circuits/bitify.circom";

// Elegant Merkle tree verification matching our Rust implementation
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
        // Convert index to bit for path selection
        indexBits[i] = Num2Bits(1);
        indexBits[i].in <== pathIndices[i];
        
        // Hash with Poseidon matching Light Protocol's new_circom(2)
        hashers[i] = Poseidon(2);
        hashers[i].inputs[0] <== currentHash[i] * (1 - indexBits[i].out[0]) + pathElements[i] * indexBits[i].out[0];
        hashers[i].inputs[1] <== pathElements[i] * (1 - indexBits[i].out[0]) + currentHash[i] * indexBits[i].out[0];
        
        currentHash[i + 1] <== hashers[i].out;
    }

    // Verify the computed root matches
    root === currentHash[levels];
}

// Main withdrawal circuit - elegant and minimal like original Tornado Cash
template Withdraw(levels) {
    // Public inputs (what the contract sees)
    signal input root;
    signal input nullifierHash;
    signal input recipient; // Field element not address
    signal input relayer;   // Field element not address  
    signal input fee;
    signal input refund;

    // Private inputs (user's secrets)
    signal input nullifier;
    signal input secret;
    signal input pathElements[levels];
    signal input pathIndices[levels];

    // Compute commitment = Poseidon(nullifier, secret)
    component commitmentHasher = Poseidon(2);
    commitmentHasher.inputs[0] <== nullifier;
    commitmentHasher.inputs[1] <== secret;

    // Compute nullifier hash = Poseidon(nullifier) 
    // Using Poseidon(1) to match Light Protocol's new_circom(1)
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

    // Add dummy constraints to prevent tampering
    signal recipientSquare;
    signal relayerSquare;
    signal feeSquare;
    signal refundSquare;
    
    recipientSquare <== recipient * recipient;
    relayerSquare <== relayer * relayer;
    feeSquare <== fee * fee;
    refundSquare <== refund * refund;
}

// Instantiate with 20 levels matching our Rust Merkle tree
component main {public [root, nullifierHash, recipient, relayer, fee, refund]} = Withdraw(20);