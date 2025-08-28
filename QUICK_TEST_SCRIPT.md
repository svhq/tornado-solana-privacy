# 🚀 Quick Test Script - 5 Minute Validation

## The Absolute Minimum Test

### Step 1: Check Everything Compiles
```bash
cd tornado_solana
cargo check --lib
```

**Expected**: No errors (warnings OK)

### Step 2: Run Core Tests
```bash
# Test the nullifier PDA solution
cargo test --lib nullifier_pda_tests -- --nocapture

# Test Merkle tree
cargo test --lib merkle_tree::tests -- --nocapture

# Test proof verification (REQUIRES vk_bytes.json - no mock fallback)
cargo test --lib real_proof_tests -- --nocapture
```

### Step 3: Quick Integration Test
```bash
# This will:
# 1. Start local validator
# 2. Deploy program
# 3. Run basic deposit/withdraw simulation

# With compute unit logging:
anchor test --skip-build -- --nocapture

# Look for lines like:
# "Program consumed 150234 of 200000 compute units"
```

---

## 🎯 What Success Looks Like

### Console Output Should Show:
```
running 5 tests
test nullifier_pda_tests::test_nullifier_pda_derivation ... ok
test nullifier_pda_tests::test_different_nullifiers_different_pdas ... ok
test nullifier_pda_tests::test_performance_comparison ... ok
test merkle_tree::tests::test_merkle_tree_initialization ... ok
test merkle_tree::tests::test_insert_and_root_update ... ok

test result: ok. 5 passed; 0 failed
```

### Key Indicators:
✅ **Nullifier PDA**: "Different nullifiers must create different PDAs"
✅ **Performance**: "PDA derivation: X ns" (should be < 1ms)
✅ **Merkle Tree**: "Root updated correctly"
✅ **No Panics**: No "thread panicked" messages

---

## 🔴 If Tests Fail

### Common Issues & Quick Fixes:

**1. Program ID Mismatch**
```
Error: Program ID mismatch
Fix: Check lib.rs matches Anchor.toml
```

**2. Missing Dependencies**
```
Error: could not find `anchor_lang`
Fix: cargo add anchor-lang
```

**3. Circuit Files Missing**
```
Error: vk_bytes.json not found
Fix: MUST have real circuit files - no mock fallback!
     cd circuits && npm run build
     This creates vk_bytes.json needed for stored-VK path
```

---

## 📊 Performance Check

Run this to see compute units:
```bash
cargo test --lib test_performance_comparison -- --nocapture
```

You should see:
```
Vec lookup (10k elements): ~XXXms
PDA derivation: ~Xμs
Speedup: XXXx
```

**Good**: PDA is 100x+ faster
**Bad**: If PDA is slower, something's wrong

---

## ✅ Minimum Viable Test Checklist

Before proceeding to devnet:

- [ ] Code compiles without errors
- [ ] Nullifier PDA tests pass
- [ ] Merkle tree tests pass  
- [ ] No "thread panicked" errors
- [ ] PDA faster than Vec lookups

If all boxes checked → **Ready for next phase!**

---

## 🎉 Success Message

If you see this, you're good:
```
test result: ok. X passed; 0 failed; 0 ignored
```

**Congratulations! The core system works!** 🚀