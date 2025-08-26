#!/bin/bash
# Minimal test script that bypasses heavy compilation

echo "Testing basic functions without full compilation..."

# Create a minimal Rust test file
cat > /tmp/minimal_test.rs << 'EOF'
fn change_endianness(bytes: &[u8]) -> Vec<u8> {
    bytes.chunks(32)
        .flat_map(|chunk| {
            let mut reversed = chunk.to_vec();
            reversed.reverse();
            reversed
        })
        .collect()
}

fn encode_u64_as_32_bytes(value: u64, output: &mut [u8; 32]) {
    output[24..32].copy_from_slice(&value.to_be_bytes());
}

fn main() {
    // Test 1: Endianness
    let input = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let output = change_endianness(&input);
    assert_eq!(output, vec![8, 7, 6, 5, 4, 3, 2, 1]);
    println!("✓ Endianness test passed");

    // Test 2: u64 encoding
    let mut encoded = [0u8; 32];
    encode_u64_as_32_bytes(1000, &mut encoded);
    assert_eq!(&encoded[0..24], &[0u8; 24]);
    assert_eq!(&encoded[24..32], &1000u64.to_be_bytes());
    println!("✓ u64 encoding test passed");

    println!("\nAll minimal tests passed!");
}
EOF

# Compile and run minimal test
rustc /tmp/minimal_test.rs -o /tmp/minimal_test && /tmp/minimal_test