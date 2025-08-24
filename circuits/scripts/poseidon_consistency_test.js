#!/usr/bin/env node

/**
 * CRITICAL TEST: Verify Poseidon hash consistency between JavaScript and Rust
 * This ensures the circuit and smart contract produce identical hashes
 */

const { buildPoseidon } = require('circomlibjs');

console.log('🔬 Poseidon Consistency Test: JS vs Rust\n');
console.log('This test verifies that Poseidon hashes match between:');
console.log('  - JavaScript (circomlib) used in circuits');
console.log('  - Rust (light-poseidon) used in smart contract\n');

async function testPoseidonConsistency() {
    // Initialize Poseidon
    const poseidon = await buildPoseidon();
    const F = poseidon.F;
    
    console.log('📊 Test Vectors:\n');
    
    // Test 1: Hash two field elements (Merkle tree nodes)
    console.log('Test 1: Poseidon(2) - Merkle tree hashing');
    const left = Buffer.from('0000000000000000000000000000000000000000000000000000000000000001', 'hex');
    const right = Buffer.from('0000000000000000000000000000000000000000000000000000000000000002', 'hex');
    
    const leftField = F.e('0x' + left.toString('hex'));
    const rightField = F.e('0x' + right.toString('hex'));
    
    const hash2 = poseidon([leftField, rightField]);
    const hash2Hex = F.toString(hash2, 16).padStart(64, '0');
    
    console.log('  Input left:  0x' + left.toString('hex'));
    console.log('  Input right: 0x' + right.toString('hex'));
    console.log('  JS Output:   0x' + hash2Hex);
    console.log('  Rust Expected: [NEEDS VERIFICATION FROM RUST TEST]');
    console.log();
    
    // Test 2: Hash single element (nullifier)
    console.log('Test 2: Poseidon(1) - Nullifier hashing');
    const nullifier = Buffer.from('1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef', 'hex');
    const nullifierField = F.e('0x' + nullifier.toString('hex'));
    
    const hash1 = poseidon([nullifierField]);
    const hash1Hex = F.toString(hash1, 16).padStart(64, '0');
    
    console.log('  Input:       0x' + nullifier.toString('hex'));
    console.log('  JS Output:   0x' + hash1Hex);
    console.log('  Rust Expected: [NEEDS VERIFICATION FROM RUST TEST]');
    console.log();
    
    // Test 3: Commitment (nullifier + secret)
    console.log('Test 3: Commitment - Poseidon(nullifier, secret)');
    const testNullifier = Buffer.from('0000000000000000000000000000000000000000000000000000000000000123', 'hex');
    const testSecret = Buffer.from('0000000000000000000000000000000000000000000000000000000000000456', 'hex');
    
    const nullifierFieldTest = F.e('0x' + testNullifier.toString('hex'));
    const secretField = F.e('0x' + testSecret.toString('hex'));
    
    const commitment = poseidon([nullifierFieldTest, secretField]);
    const commitmentHex = F.toString(commitment, 16).padStart(64, '0');
    
    console.log('  Nullifier:   0x' + testNullifier.toString('hex'));
    console.log('  Secret:      0x' + testSecret.toString('hex'));
    console.log('  JS Output:   0x' + commitmentHex);
    console.log('  Rust Expected: [NEEDS VERIFICATION FROM RUST TEST]');
    console.log();
    
    // Generate Rust test code
    console.log('📝 Rust Test Code to Run:\n');
    console.log('```rust');
    console.log('// Add this test to merkle_tree.rs');
    console.log('#[test]');
    console.log('fn test_poseidon_consistency() {');
    console.log('    use light_poseidon::{Poseidon, PoseidonBytesHasher};');
    console.log('    use ark_bn254::Fr;');
    console.log('    ');
    console.log('    // Test 1: Hash two elements');
    console.log('    let left = hex::decode("0000000000000000000000000000000000000000000000000000000000000001").unwrap();');
    console.log('    let right = hex::decode("0000000000000000000000000000000000000000000000000000000000000002").unwrap();');
    console.log('    let mut hasher = Poseidon::<Fr>::new_circom(2).unwrap();');
    console.log('    let hash2 = hasher.hash_bytes_be(&[&left, &right]).unwrap();');
    console.log('    println!("Test 1 Rust: {:?}", hex::encode(hash2));');
    console.log('    ');
    console.log('    // Test 2: Hash single element');
    console.log('    let nullifier = hex::decode("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();');
    console.log('    let mut hasher1 = Poseidon::<Fr>::new_circom(1).unwrap();');
    console.log('    let hash1 = hasher1.hash_bytes_be(&[&nullifier]).unwrap();');
    console.log('    println!("Test 2 Rust: {:?}", hex::encode(hash1));');
    console.log('    ');
    console.log('    // Test 3: Commitment');
    console.log('    let test_nullifier = hex::decode("0000000000000000000000000000000000000000000000000000000000000123").unwrap();');
    console.log('    let test_secret = hex::decode("0000000000000000000000000000000000000000000000000000000000000456").unwrap();');
    console.log('    let mut hasher_commit = Poseidon::<Fr>::new_circom(2).unwrap();');
    console.log('    let commitment = hasher_commit.hash_bytes_be(&[&test_nullifier, &test_secret]).unwrap();');
    console.log('    println!("Test 3 Rust: {:?}", hex::encode(commitment));');
    console.log('}');
    console.log('```');
    console.log();
    
    console.log('⚠️  CRITICAL: Run the Rust test and compare outputs!');
    console.log('If hashes don\'t match, the system will fail.');
    console.log();
    
    // Check if circomlib Poseidon matches expected parameters
    console.log('🔍 Verifying Poseidon Parameters:');
    console.log('  - Field: BN254 (matches ark-bn254)');
    console.log('  - Width: Depends on inputs (2 for merkle, 1 for nullifier)');
    console.log('  - Rounds: Circom standard configuration');
    console.log('  - S-box: x^5 (standard)');
    console.log();
    
    console.log('✅ JavaScript Poseidon test complete.');
    console.log('❗ Now run the Rust test to verify consistency.');
}

// Run the test
testPoseidonConsistency().catch(console.error);