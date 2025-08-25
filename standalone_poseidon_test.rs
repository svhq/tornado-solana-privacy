// Standalone Poseidon consistency test
// Run with: rustc standalone_poseidon_test.rs && ./standalone_poseidon_test

use std::process::Command;

fn main() {
    println!("🔬 Poseidon Consistency Test (Standalone)\n");
    
    // Test vectors from JavaScript
    println!("Expected JavaScript outputs:");
    println!("Test 1 (Merkle): 0x115cc0f5e7d690413df64c6b9662e9cf2a3617f2743245519e19607a4417189a");
    println!("Test 2 (Nullifier): 0x239edbf1e6b4f5646471d24e63b1ab7992897e0ecefa6b565302f64fe1e49117");
    println!("Test 3 (Commitment): 0x0e7a333190bcbb4f654dbefca544b4a2b0644d05dce3fdc11e6df0b6e4fa57d4\n");
    
    println!("To verify Rust matches, create a simple test:");
    println!("1. Create a new Rust project: cargo new poseidon_test");
    println!("2. Add to Cargo.toml:");
    println!("   [dependencies]");
    println!("   light-poseidon = \"0.2.0\"");
    println!("   ark-bn254 = \"0.4.0\"");
    println!("   hex = \"0.4\"\n");
    
    println!("3. Add this test code to src/main.rs:");
    println!("----------------------------------------");
    print_test_code();
    println!("----------------------------------------\n");
    
    println!("4. Run: cargo run");
    println!("\nIf the outputs match, Poseidon is consistent! ✅");
}

fn print_test_code() {
    println!(r#"use light_poseidon::{{Poseidon, PoseidonBytesHasher}};
use ark_bn254::Fr;

fn main() {{
    println!("Rust Poseidon Test:\n");
    
    // Test 1: Hash two elements
    let left: [u8; 32] = [0; 31].iter().chain(&[1]).cloned().collect::<Vec<_>>().try_into().unwrap();
    let right: [u8; 32] = [0; 31].iter().chain(&[2]).cloned().collect::<Vec<_>>().try_into().unwrap();
    
    let mut hasher = Poseidon::<Fr>::new_circom(2).unwrap();
    let hash2 = hasher.hash_bytes_be(&[&left, &right]).unwrap();
    println!("Test 1: 0x{{}}", hex::encode(hash2));
    
    // Test 2: Hash single element  
    let nullifier = hex::decode("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
    let nullifier_bytes: [u8; 32] = nullifier.try_into().unwrap();
    
    let mut hasher1 = Poseidon::<Fr>::new_circom(1).unwrap();
    let hash1 = hasher1.hash_bytes_be(&[&nullifier_bytes]).unwrap();
    println!("Test 2: 0x{{}}", hex::encode(hash1));
    
    // Test 3: Commitment
    let test_nullifier: [u8; 32] = [0; 30].iter().chain(&[0x01, 0x23]).cloned().collect::<Vec<_>>().try_into().unwrap();
    let test_secret: [u8; 32] = [0; 30].iter().chain(&[0x04, 0x56]).cloned().collect::<Vec<_>>().try_into().unwrap();
    
    let mut hasher_commit = Poseidon::<Fr>::new_circom(2).unwrap();
    let commitment = hasher_commit.hash_bytes_be(&[&test_nullifier, &test_secret]).unwrap();
    println!("Test 3: 0x{{}}", hex::encode(commitment));
}}"#);
}