# Tornado Solana - Privacy Protocol for Solana

A direct, elegant translation of Tornado Cash to Rust/Anchor for the Solana blockchain. This implementation maintains the beautiful simplicity of the original ~200 lines of core logic while adapting to Solana's architecture.

## Core Features

- **Anonymous Deposits**: Deposit SOL with a commitment (hash of nullifier + secret)
- **Private Withdrawals**: Withdraw to any address with zero-knowledge proof
- **Merkle Tree with History**: 20-level tree supporting 1,048,576 deposits
- **Double-Spend Prevention**: Nullifier tracking prevents note reuse
- **Fixed Denominations**: Pool-based approach for strong anonymity sets

## Implementation Details

### Files Structure
```
tornado_solana/
├── programs/tornado_solana/
│   ├── src/
│   │   ├── lib.rs           # Main program logic (~190 lines)
│   │   └── merkle_tree.rs   # Merkle tree implementation (~120 lines)
│   └── Cargo.toml
├── tests/
│   └── tornado_solana.ts    # Comprehensive test suite
├── Anchor.toml
└── package.json
```

### Core Components

1. **TornadoState Account**
   - Stores denomination, merkle tree, roots history
   - Tracks nullifiers and commitments
   - Maintains 30 historical roots for proof flexibility

2. **Merkle Tree Module**
   - 20-level binary tree
   - Efficient sparse storage using filled_subtrees
   - Poseidon hashing (ZK-friendly, circuit-compatible)

3. **Instructions**
   - `initialize(denomination)` - Deploy new pool
   - `deposit(commitment)` - Anonymous deposit
   - `withdraw(proof, root, nullifier_hash, recipient, relayer, fee, refund)` - Private withdrawal

## Current Status

✅ **Completed**:
- Core Tornado Cash logic translated to Rust/Anchor
- Merkle tree with history implementation
- Deposit and withdrawal instructions
- Nullifier and commitment tracking
- Basic test suite
- ✅ Poseidon hash integration (using Light Protocol's light-poseidon v0.2.0)
- Comprehensive test coverage (40+ test cases)

⚠️ **TODO for Production**:
- [ ] Integrate Light Protocol's groth16-solana for real proof verification
- [ ] Add circuit compilation and proving key generation
- [ ] Implement relayer infrastructure
- [ ] Add multiple denomination support
- [ ] Optimize account size for Solana's limits

## Usage

### Build
```bash
anchor build
```

### Test
```bash
anchor test
```

### Deploy to Devnet
```bash
anchor deploy --provider.cluster devnet
```

## How It Works

1. **Deposit Phase**:
   - User generates random `nullifier` and `secret`
   - Computes `commitment = Hash(nullifier || secret)`
   - Sends SOL to pool with commitment
   - Commitment added to Merkle tree

2. **Withdrawal Phase**:
   - User generates zkSNARK proof of knowledge of (nullifier, secret)
   - Proof shows commitment exists in tree without revealing which one
   - Submits `nullifierHash = Hash(nullifier)` to prevent double-spending
   - Funds sent to specified recipient

3. **Privacy Guarantees**:
   - Deposit and withdrawal are unlinkable
   - Nullifier hash doesn't reveal the commitment
   - Zero-knowledge proof hides which deposit is being withdrawn

## Security Notes

- **Proof Verification**: Currently mocked for testing. Production requires real Groth16 verification
- **Hash Function**: ✅ Now using Poseidon (light-poseidon v0.2.0) for ZK-circuit compatibility
- **Trusted Setup**: Production deployment requires a ceremony for proving/verifying keys
- **Audit**: Code should be professionally audited before mainnet deployment

## Next Steps for Production

1. **Integrate Light Protocol Libraries**:
   ```toml
   [dependencies]
   groth16-solana = "0.0.1"  # Still needed for proof verification
   light-poseidon = "0.2.0"  # ✅ Already integrated
   ```

2. **Add Circuit Implementation**:
   - Design withdraw.circom circuit
   - Generate proving and verifying keys
   - Integrate with client SDK

3. **Optimize for Solana**:
   - Account size management
   - Compute unit optimization
   - Use Program Derived Addresses for scaling

4. **Add Fuel Note System** (Innovation):
   - Fixed denomination notes for fee privacy
   - Shielded fee buffer
   - Enhanced metadata resistance

## Architecture Elegance

The implementation maintains Tornado Cash's elegant design:
- **~200 lines** of core logic (lib.rs)
- **Clean separation** between program and merkle tree
- **Minimal state** - only essential mappings
- **No admin controls** - fully decentralized
- **Immutable parameters** - trustless operation

## License

MIT