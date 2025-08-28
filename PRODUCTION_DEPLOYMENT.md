# 🚀 Production Deployment Guide

## Pre-Deployment Checklist

### 1. Update Anchor.toml Configuration

#### Current (Localnet):
```toml
[features]
seeds = true  # ✅ Already enabled for security
skip-lint = false

[programs.localnet]
tornado_solana = "ToRNaDo1111111111111111111111111111111111111"

[provider]
cluster = "Localnet"
wallet = "~/.config/solana/id.json"
```

#### For Devnet:
```toml
[programs.devnet]
tornado_solana = "YOUR_DEPLOYED_PROGRAM_ID_HERE"  # Update after deployment

[provider]
cluster = "Devnet"  # or "devnet" 
wallet = "~/.config/solana/id.json"  # Or your deployment wallet
```

#### For Mainnet:
```toml
[programs.mainnet]
tornado_solana = "YOUR_DEPLOYED_PROGRAM_ID_HERE"  # Update after deployment

[provider]
cluster = "Mainnet"  # or "mainnet-beta"
wallet = "/path/to/secure/deployment/wallet.json"  # Use secure wallet
```

---

## Deployment Steps

### Step 1: Generate New Program Keypair
```bash
# Generate a new program keypair for deployment
solana-keygen new -o target/deploy/tornado_solana-keypair.json

# Get the program ID
solana address -k target/deploy/tornado_solana-keypair.json
# Example output: 8FqckwnxHPrxxxxxxxxxxxxxxxxxxxxxxxxxxxxxN

# Update lib.rs with the new program ID
# Replace: declare_id!("ToRNaDo1111111111111111111111111111111111111");
# With:    declare_id!("8FqckwnxHPrxxxxxxxxxxxxxxxxxxxxxxxxxxxxxN");
```

### Step 2: Build for Target Network
```bash
# Set cluster
solana config set --url https://api.devnet.solana.com  # For devnet
# OR
solana config set --url https://api.mainnet-beta.solana.com  # For mainnet

# Build the program
anchor build

# Verify the program ID matches
anchor keys list
```

### Step 3: Deploy Program
```bash
# For devnet (free)
anchor deploy --provider.cluster devnet

# For mainnet (requires SOL for deployment)
anchor deploy --provider.cluster mainnet

# Note the deployed program ID
```

### Step 4: Update Configuration Files

1. **Update Anchor.toml** with the deployed program ID
2. **Update lib.rs** declare_id! macro
3. **Rebuild** to ensure consistency:
```bash
anchor build
```

### Step 5: Initialize On-Chain State
```bash
# Run initialization script
anchor run initialize

# Or manually:
ts-node scripts/initialize.ts
```

### Step 6: Verify Deployment
```bash
# Check program is deployed
solana program show <PROGRAM_ID>

# Run tests against deployed program
anchor test --skip-local-validator --provider.cluster devnet
```

---

## Security Checklist

### Before Mainnet:
- [ ] Professional security audit completed
- [ ] Trusted setup ceremony performed for verifying keys
- [ ] All tests passing on devnet
- [ ] Load testing completed (10k+ operations)
- [ ] Compute units verified (<200k per instruction)
- [ ] Multi-sig wallet for deployment
- [ ] Upgrade authority properly configured
- [ ] Emergency pause mechanism tested

### Configuration Security:
- [ ] `seeds = true` enabled in Anchor.toml ✅
- [ ] All PDAs use seeds constraints ✅
- [ ] Manual PDA validation in place ✅
- [ ] No hardcoded secrets or keys
- [ ] Verifying key from trusted setup

---

## Environment Variables

### Create `.env` file (DO NOT COMMIT):
```env
# Network
CLUSTER=devnet  # or mainnet-beta
RPC_URL=https://api.devnet.solana.com

# Program
PROGRAM_ID=YOUR_DEPLOYED_PROGRAM_ID

# Wallet (for scripts, not deployment)
WALLET_PATH=/path/to/wallet.json

# Optional: Custom RPC
CUSTOM_RPC=https://your-rpc-provider.com
```

---

## Post-Deployment Monitoring

### Key Metrics:
- Transaction success rate
- Average compute units per instruction
- Nullifier PDA creation rate
- Vault balance
- Gas costs

### Monitoring Commands:
```bash
# Check program logs
solana logs <PROGRAM_ID> --url <RPC_URL>

# Monitor transactions
solana confirm -v <TX_SIGNATURE>

# Check account balance
solana balance <VAULT_PDA_ADDRESS>
```

---

## Rollback Plan

### If Issues Detected:
1. **Pause deposits** - Disable new deposits via feature flag
2. **Allow withdrawals** - Let users withdraw funds
3. **Deploy fix** - Update and redeploy program
4. **Migrate state** - If needed, migrate to new program

### Emergency Contacts:
- Security Team: [CONTACT]
- DevOps Team: [CONTACT]  
- Audit Firm: [CONTACT]

---

## Common Issues & Solutions

### Issue: Program ID Mismatch
**Solution**: Ensure declare_id! in lib.rs matches Anchor.toml

### Issue: Insufficient SOL for deployment
**Solution**: 
- Devnet: Use `solana airdrop`
- Mainnet: Fund deployment wallet with ~5-10 SOL

### Issue: Compute units exceeded
**Solution**: Verify nullifier PDA implementation is active (not Vec)

### Issue: Seeds constraint errors
**Solution**: Ensure `seeds = true` in Anchor.toml and all PDAs have seeds/bump

---

## Final Checklist

Before going live:
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Team trained on operations
- [ ] Monitoring in place
- [ ] Incident response plan ready
- [ ] Community notified
- [ ] Audit report published

---

## Quick Reference

### Commands:
```bash
# Build
anchor build

# Test
anchor test

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Deploy to mainnet
anchor deploy --provider.cluster mainnet

# Verify deployment
solana program show <PROGRAM_ID>
```

### Important Files:
- `Anchor.toml` - Network configuration
- `lib.rs` - Program ID declaration
- `target/deploy/` - Deployment artifacts
- `.env` - Environment variables (git-ignored)

---

**Remember**: Always test on devnet before mainnet deployment!