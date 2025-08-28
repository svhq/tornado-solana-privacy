# 🧪 Local Testing Plan - Tornado Solana

## Executive Summary

We'll run a complete test suite locally using Anchor's test validator, verifying all security properties without needing devnet/testnet.

---

## ✅ Prerequisites Check

### Circuit Artifacts (REQUIRED - NO MOCK FALLBACK)
```
✅ circuits/build/withdraw_final.zkey     - Proving key
✅ circuits/build/verification_key.json   - Human-readable VK
✅ circuits/build/vk_bytes.json          - Binary VK for Rust (MUST EXIST)
```

⚠️ **IMPORTANT**: Run `initialize(vk_bytes.json)` as Step 1 to store the VK on-chain before any withdraw tests. Tests will fail if VK files are missing - this is intentional to ensure we test the stored-VK path.

### Dependencies Needed
```bash
# Install if not present
npm install          # Node dependencies
cargo build-sbf      # Solana BPF toolchain
```

---

## 🎯 Test Strategy - First Principles

### Level 1: Component Tests (Unit)
Test each piece in isolation:
1. **Merkle Tree** - Insert, root calculation, proof generation
2. **Poseidon Hash** - Consistency with circuit
3. **Nullifier PDA** - Derivation and uniqueness
4. **Vault PDA** - Proper derivation and validation

### Level 2: Integration Tests
Test components working together:
1. **Deposit Flow** - Funds → Vault, Commitment → Tree
2. **Withdrawal Flow** - Proof → Verification → Payout
3. **Double-Spend Prevention** - Nullifier PDA blocks reuse
4. **Relayer Security** - Fee validation

### Level 3: End-to-End Test
Complete user journey with verification:
```
1. Initialize pool with vk_bytes.json content
2. Deposit 1 SOL
   a. Assert vault received funds
   b. Fetch on-chain root
   c. Build off-chain tree with same zero-chain
   d. Assert roots match (CRITICAL for proof generation)
3. Generate proof (off-chain)
4. Withdraw to new address
   a. Assert recipient credited
   b. Verify nullifier PDA created
   c. Log who paid PDA rent (relayer vs user)
5. Attempt double-spend with same nullifier
   a. MUST fail at PDA init
   b. Assert error is "account already exists"
```

---

## 🚀 Quick Test Command

### The One-Liner (as consultant suggested)
```bash
anchor test
```

### What This Does
1. Starts local validator
2. Deploys program
3. Runs all tests
4. Shows results

---

## 📝 Step-by-Step Test Execution

### Step 1: Build Program
```bash
# From tornado_solana directory
anchor build

# Expected output:
# - target/deploy/tornado_solana.so
# - target/idl/tornado_solana.json
```

### Step 2: Run Rust Unit Tests
```bash
# Test individual components
cargo test --lib merkle_tree
cargo test --lib poseidon_test
cargo test --lib nullifier_pda_test
```

### Step 3: Run Integration Tests
```bash
# Test with local validator
anchor test --skip-build

# Or specific test file:
anchor test --skip-build tests/tornado_solana.ts
```

### Step 4: Verify Security Properties
```bash
# Double-spend prevention test
cargo test --lib test_nullifier_prevents_double_spend

# Recipient safety test
cargo test --lib test_recipient_executable_check
```

---

## 🔍 What to Look For

### ✅ Good Signs
```
✓ All tests passing
✓ "Real proof verified successfully!"
✓ "Double-spend prevented"
✓ Compute units: < 200,000
```

### ❌ Red Flags
```
✗ "Program ID mismatch" → Check Anchor.toml
✗ "Nullifier already exists" → Good (security working)
✗ "InvalidProof" → Check VK or proof generation
✗ Compute units > 200k → Optimization needed
```

---

## 📊 Test Coverage Matrix

| Component | Unit Test | Integration | E2E | Security |
|-----------|-----------|------------|-----|----------|
| Merkle Tree | ✅ | ✅ | ✅ | ✅ |
| Poseidon | ✅ | ✅ | ✅ | ✅ |
| Nullifier PDA | ✅ | ✅ | ✅ | ✅ |
| Vault PDA | ✅ | ✅ | ✅ | ✅ |
| Proof Verify | ✅ | ✅ | ✅ | ✅ |
| Double-Spend | - | ✅ | ✅ | ✅ |
| Relayer | - | ✅ | ✅ | ✅ |

---

## 🎯 Minimal Smoke Test

If you want the absolute minimum test:

```bash
# 1. Build
anchor build

# 2. Test one core flow
cargo test --lib test_real_proof_verification

# 3. If that passes, system core works
```

---

## 🔧 Troubleshooting

### Issue: Program ID Mismatch
```bash
# Get correct ID
solana-keygen pubkey target/deploy/tornado_solana-keypair.json

# Update both:
# - lib.rs: declare_id!("...")
# - Anchor.toml: tornado_solana = "..."
```

### Issue: Missing Dependencies
```bash
npm install
cargo install anchor-cli
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
```

### Issue: Circuit Files Missing
```bash
cd circuits
npm install
npm run build  # Or follow setup.js, compile.js, generate_vk.js
```

---

## 📈 Performance Benchmarks

### How to Record ACTUAL Compute Units
```bash
# Run with --nocapture to see logs
anchor test -- --nocapture

# Look for lines like:
# "Program ToRNaDo111... consumed 152341 of 200000 compute units"
```

### Record From Logs (Not Estimates)
| Operation | Actual CU (from logs) | Target |
|-----------|----------------------|--------|
| Initialize | _____ (record here) | <50k |
| Deposit | _____ (record here) | <80k |
| Withdraw | _____ (record here) | <200k |
| Nullifier Check | _____ (record here) | <5k |

### Optimization Targets
- Total CU per withdraw: < 200k ✅
- Proof verification: < 100k ✅
- Nullifier PDA: < 5k ✅

---

## ✅ Success Criteria

The system is ready when:
1. All tests pass
2. Compute units under limits
3. Security properties verified
4. No error logs

---

## 🚦 Go/No-Go Decision

### GO (Ready for Devnet) if:
- ✅ All unit tests pass
- ✅ Integration tests pass
- ✅ Double-spend prevented
- ✅ CU < 200k

### NO-GO (Needs fixes) if:
- ❌ Any security test fails
- ❌ CU > 200k
- ❌ Proof verification fails
- ❌ Program doesn't deploy

---

## 📝 Test Log Template

```
Date: [DATE]
Tester: [NAME]
Version: v0.1.0-polish-complete

Results:
[ ] Program builds successfully
[ ] Unit tests pass (X/Y)
[ ] Integration tests pass (X/Y)
[ ] Security tests pass (X/Y)
[ ] VK loaded from vk_bytes.json: Yes/No
[ ] Off-chain/on-chain roots match: Yes/No
[ ] Double-spend blocked: Yes/No

Compute Units (from logs):
- Initialize: _____ CU
- Deposit: _____ CU  
- Withdraw: _____ CU
- Nullifier PDA: _____ CU

Rent Payer: [relayer/recipient]
Ready for devnet: Yes/No

Notes:
[Any issues or observations]
```

---

## 🎉 Next Steps After Success

1. Deploy to devnet
2. Run same tests on devnet
3. Community testing
4. Security audit
5. Mainnet deployment

---

**Remember**: A failed test is a successful security check!