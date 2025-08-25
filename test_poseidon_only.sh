#!/bin/bash

echo "Testing Poseidon consistency ONLY (skipping Groth16)..."
echo ""

# Just test the Poseidon module directly
cd /mnt/c/Users/cc/claude\ code\ privacy\ solana/tornado_solana/programs/tornado_solana

# Run only the Poseidon test, not the full program
cargo test --lib poseidon_test::poseidon_consistency_tests::test_poseidon_consistency -- --nocapture 2>/dev/null || \
cargo test poseidon_consistency 2>/dev/null || \
echo "Need to compile the Poseidon test separately"

# If that doesn't work, compile just the test file
rustc --test src/poseidon_test.rs -L target/debug/deps 2>/dev/null && ./poseidon_test