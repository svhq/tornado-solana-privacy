# Tornado Solana Scaling Solution - O(n) to O(1) Nullifier Lookups

## Problem Statement

The current implementation uses Vec<[u8; 32]> for storing nullifiers and commitments with O(n) lookup operations:
```rust
pub nullifier_hashes: Vec<[u8; 32]>  // Line 409 in lib.rs
!tornado_state.nullifier_hashes.contains(&nullifier_hash)  // Line 140
```

### Scaling Limits
- **At 10k nullifiers**: ~300-500k compute units (approaching limit)
- **At 50k+ nullifiers**: EXCEEDS SOLANA LIMIT - program fails completely
- **Each .contains() call**: O(n) linear search through entire Vec

## Temporary Safeguard (Implemented)

Added capacity checks to prevent compute unit overflow:
```rust
// Constants added to lib.rs
pub const MAX_NULLIFIERS_PER_POOL: usize = 10_000;
pub const MAX_COMMITMENTS_PER_POOL: usize = 10_000;

// Check in withdraw function
require!(
    tornado_state.nullifier_hashes.len() < MAX_NULLIFIERS_PER_POOL,
    TornadoError::PoolFull
);
```

**Status**: ✅ Prevents immediate failure, but limits pool capacity

## Production Solution: PDA Map Pattern

### Architecture Overview

Replace Vec storage with individual PDA accounts for each nullifier:

```rust
// Each nullifier gets its own PDA account
pub struct NullifierRecord {
    pub spent: bool,        // Always true when account exists
    pub spent_at: i64,      // Timestamp for analytics
    pub spent_slot: u64,     // Slot number for ordering
}

// O(1) lookup by deriving PDA address deterministically
let (nullifier_pda, bump) = Pubkey::find_program_address(
    &[b"nullifier", nullifier_hash.as_ref()],
    &program_id,
);
```

### Why This Solves the Problem

1. **O(1) Lookups**: Check existence by trying to load PDA account
2. **Unlimited Scale**: No Vec to iterate through
3. **Rent Efficient**: Only pay for nullifiers that exist
4. **Deterministic**: PDA address derived from nullifier hash

## Implementation Plan

### Phase 1: Data Structures

```rust
// New account structures
#[account]
pub struct NullifierRecord {
    pub nullifier_hash: [u8; 32],  // The nullifier this record represents
    pub spent_at: i64,              // Unix timestamp
    pub spent_slot: u64,            // Solana slot number
    pub withdrawal_tx: Pubkey,      // Transaction that spent this nullifier
}

#[account]
pub struct CommitmentRecord {
    pub commitment: [u8; 32],       // The commitment
    pub deposited_at: i64,          // Unix timestamp
    pub deposited_slot: u64,        // Solana slot number
    pub leaf_index: u32,            // Position in merkle tree
}
```

### Phase 2: Modified Instructions

#### Withdraw with PDA Check
```rust
#[derive(Accounts)]
pub struct WithdrawV2<'info> {
    #[account(mut)]
    pub tornado_state: Account<'info, TornadoState>,
    
    // Try to create the nullifier PDA - fails if it already exists
    #[account(
        init_if_needed,
        payer = relayer,
        space = 8 + 32 + 8 + 8 + 32, // discriminator + fields
        seeds = [b"nullifier", nullifier_hash.as_ref()],
        bump
    )]
    pub nullifier_record: Account<'info, NullifierRecord>,
    
    // ... other accounts
}

pub fn withdraw_v2(ctx: Context<WithdrawV2>, ...) -> Result<()> {
    // If nullifier_record already exists, init_if_needed will fail
    // This replaces the O(n) .contains() check
    
    // Record nullifier spending
    ctx.accounts.nullifier_record.nullifier_hash = nullifier_hash;
    ctx.accounts.nullifier_record.spent_at = Clock::get()?.unix_timestamp;
    ctx.accounts.nullifier_record.spent_slot = Clock::get()?.slot;
    ctx.accounts.nullifier_record.withdrawal_tx = ctx.accounts.relayer.key();
    
    // Continue with withdrawal...
}
```

#### Deposit with PDA Creation
```rust
#[derive(Accounts)]
pub struct DepositV2<'info> {
    #[account(mut)]
    pub tornado_state: Account<'info, TornadoState>,
    
    // Create commitment record
    #[account(
        init,
        payer = depositor,
        space = 8 + 32 + 8 + 8 + 4,
        seeds = [b"commitment", commitment.as_ref()],
        bump
    )]
    pub commitment_record: Account<'info, CommitmentRecord>,
    
    // ... other accounts
}
```

### Phase 3: Migration Strategy

Create migration instruction to move existing nullifiers:

```rust
pub fn migrate_nullifiers_batch(
    ctx: Context<MigrateNullifiers>,
    start_index: usize,
    batch_size: usize
) -> Result<()> {
    let tornado_state = &ctx.accounts.tornado_state;
    let end_index = (start_index + batch_size).min(tornado_state.nullifier_hashes.len());
    
    for i in start_index..end_index {
        let nullifier_hash = tornado_state.nullifier_hashes[i];
        
        // Create PDA account for this nullifier
        // (would need to pass in accounts for each)
        // This is pseudocode - real implementation needs account array
    }
    
    // After all batches migrated, can remove Vec from state
    Ok(())
}
```

## Performance Comparison

| Metric | Current (Vec) | PDA Map |
|--------|--------------|---------|
| Lookup Complexity | O(n) | O(1) |
| Compute Units @ 10k | ~300-500k | ~5k |
| Compute Units @ 100k | FAILS | ~5k |
| Storage Cost | All in one account | Per-nullifier rent |
| Max Capacity | ~10k nullifiers | Unlimited |

## Cost Analysis

### Current Vec Storage
- Single account holds all nullifiers
- Account size: ~320KB at 10k nullifiers
- Rent: ~2.5 SOL

### PDA Map Storage  
- Per nullifier: ~100 bytes
- Rent per nullifier: ~0.002 SOL
- Total at 10k: ~20 SOL (higher but scales infinitely)

### Trade-off
- Higher storage cost for unlimited scalability
- Can subsidize from protocol fees
- Users pay small PDA creation fee on withdrawal

## Implementation Timeline

1. **Week 1**: Implement NullifierRecord and CommitmentRecord structures
2. **Week 2**: Create withdraw_v2 and deposit_v2 instructions
3. **Week 3**: Build migration tooling and test on devnet
4. **Week 4**: Audit changes and prepare mainnet deployment

## Testing Strategy

1. **Unit Tests**
   - Verify PDA derivation is deterministic
   - Test double-spend prevention
   - Ensure migration preserves all nullifiers

2. **Integration Tests**
   - Simulate 100k+ deposits/withdrawals
   - Measure compute units stay under 200k
   - Verify no race conditions

3. **Load Testing**
   - Parallel withdrawals with same nullifier (should fail)
   - Stress test with maximum transaction throughput
   - Monitor RPC performance with many PDA lookups

## Security Considerations

1. **PDA Seed Design**: Use `[b"nullifier", nullifier_hash]` to ensure uniqueness
2. **Authority**: Only program can create nullifier PDAs
3. **Atomicity**: Nullifier creation and withdrawal must be atomic
4. **Migration Safety**: Keep Vec during migration period as backup

## Alternative Approaches Considered

### 1. Bloom Filter
- Pros: Compact, O(1) operations
- Cons: False positives possible, not suitable for financial system

### 2. Off-chain Index with Proofs
- Pros: Minimal on-chain storage
- Cons: Requires trusted indexer or complex proof system

### 3. Sharded Vecs
- Pros: Simpler than PDA map
- Cons: Still O(n/k) where k is shard count

### 4. HashMap in BPF (Not Viable)
- Solana BPF doesn't support dynamic allocations
- Would require custom implementation

## Recommendation

**Implement PDA Map pattern for production:**
- Proven pattern used by major Solana protocols
- True O(1) performance at any scale
- No compute unit concerns
- Worth the additional storage cost for unlimited scalability

**Keep temporary safeguards until migration:**
- MAX_NULLIFIERS check prevents overflow
- Gives time for proper PDA implementation
- Can increase limit if needed before migration

## Code References

- Current O(n) check: `lib.rs:140`
- Nullifier storage: `lib.rs:409`
- Temporary safeguard: `lib.rs:143-147` (after implementation)
- Constants: `lib.rs:314-315` (MAX_NULLIFIERS_PER_POOL)

## Next Steps

1. ✅ Temporary safeguard (COMPLETE)
2. ⏳ Design PDA structures (THIS DOCUMENT)
3. 🔜 Implement new instructions
4. 🔜 Create migration plan
5. 🔜 Test on devnet
6. 🔜 Audit and deploy

---

**Critical Note**: The temporary safeguard is sufficient for testnet but MUST implement PDA solution before mainnet to avoid capacity limits.