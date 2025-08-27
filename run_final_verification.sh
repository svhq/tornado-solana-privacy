#!/bin/bash

echo "========================================"
echo "RUNNING FINAL VERIFICATION TEST"
echo "========================================"
echo ""
echo "This test will:"
echo "1. Verify real proof from withdraw_fixed.circom"
echo "2. Show all public inputs and cryptographic details"
echo "3. Estimate compute units"
echo "4. Validate error handling"
echo ""
echo "Starting test..."
echo ""

# Run the comprehensive verification test
cargo test --lib final_verification_test::comprehensive_real_proof_verification -- --nocapture --test-threads=1

echo ""
echo "========================================"
echo "TEST COMPLETE - CHECK OUTPUT ABOVE"
echo "========================================"