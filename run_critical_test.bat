@echo off
echo 🔴 CRITICAL POSEIDON CONSISTENCY TEST
echo =====================================
echo.
echo This test verifies that JavaScript and Rust produce identical Poseidon hashes.
echo If they don't match, THE SYSTEM WILL NOT WORK.
echo.

REM Check if cargo is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ ERROR: Cargo is not installed!
    echo Please install Rust: https://rustup.rs/
    exit /b 1
)

echo Step 1: Running JavaScript test...
echo -----------------------------------
cd circuits
node scripts\poseidon_consistency_test.js
cd ..

echo.
echo Step 2: Running Rust test...
echo ----------------------------
cargo test test_poseidon_consistency -- --nocapture

echo.
echo 🔍 VERIFICATION CHECKLIST:
echo -------------------------
echo [ ] Test 1: Merkle hashing matches (0x115cc0f5...4417189a)
echo [ ] Test 2: Nullifier hashing matches (0x239edbf1...e1e49117)
echo [ ] Test 3: Commitment hashing matches (0x0e7a3331...e4fa57d4)
echo.
echo If all three match: ✅ PROCEED WITH GROTH16
echo If any don't match: ❌ STOP AND DEBUG
pause