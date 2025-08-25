# Solana CLI & Anchor Installation Guide (2025)

## CTO Analysis: What You Need

### Current Situation
We have a working Tornado Cash implementation with:
- ✅ Rust code (lib.rs, merkle_tree.rs)
- ✅ Poseidon integration
- ✅ Groth16 verifier integration
- ✅ Circom circuits
- ⚠️ Need Solana CLI & Anchor to compile and deploy

## Installation Options for Windows (2025)

### Option 1: WSL (Windows Subsystem for Linux) - RECOMMENDED
**Time: 30-45 minutes**

#### Step 1: Install WSL
```powershell
# Run PowerShell as Administrator
wsl --install
# Restart computer after installation
```

#### Step 2: Install Ubuntu from Microsoft Store
- Open Microsoft Store
- Search "Ubuntu"
- Install Ubuntu 22.04 LTS or latest

#### Step 3: Open Ubuntu Terminal and Install Dependencies
```bash
# Update packages
sudo apt-get update && sudo apt-get upgrade -y

# Install build tools
sudo apt-get install -y pkg-config build-essential libudev-dev libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Solana CLI (Latest 2025 version)
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Install Anchor (Latest version)
cargo install --git https://github.com/coral-xyz/anchor avm --force
avm install latest
avm use latest

# Install Node.js for tests
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
```

#### Step 4: Verify Installation
```bash
solana --version  # Should show: solana-cli 2.x.x
anchor --version  # Should show: anchor-cli 0.31.x
cargo --version   # Should show: cargo 1.7x.x
node --version    # Should show: v20.x.x
```

### Option 2: Direct Windows Installation (More Complex)
**Not recommended** - Many compatibility issues with Windows native

### Option 3: Docker Container (Alternative)
```bash
# Use pre-built Solana development container
docker run -it --rm -v ${PWD}:/workspace solanalabs/solana:latest
```

## What Each Tool Does

### Solana CLI
- **Purpose**: Deploy programs to Solana blockchain
- **Commands we need**:
  - `solana-keygen new` - Create wallet
  - `solana airdrop` - Get test SOL
  - `solana program deploy` - Deploy our program

### Anchor Framework
- **Purpose**: Build and test Solana programs
- **Commands we need**:
  - `anchor build` - Compile Rust to Solana program
  - `anchor test` - Run integration tests
  - `anchor deploy` - Deploy to network

### Cargo (Rust)
- **Purpose**: Build Rust code
- **Already used in our project**
- Needed to run our Poseidon tests

## Quick Test After Installation

### 1. Clone and Build Our Project
```bash
# In WSL/Ubuntu terminal
cd /mnt/c/Users/cc/claude\ code\ privacy\ solana/tornado_solana

# Build the program
anchor build

# Run tests (including our Poseidon consistency test)
cargo test test_poseidon_consistency -- --nocapture
```

### 2. Run Our Critical Test
```bash
# JavaScript Poseidon test
cd circuits
npm install
node scripts/poseidon_consistency_test.js

# Rust Poseidon test (NOW WE CAN RUN IT!)
cd ..
cargo test test_poseidon_consistency
```

## CTO Recommendation

### Immediate Action Plan

1. **Install WSL + Ubuntu** (30 mins)
   - Most reliable for Solana development
   - All tools work perfectly in Linux environment

2. **Run Our Poseidon Test** (5 mins)
   - Finally verify JS/Rust hash consistency
   - Confirm system integrity

3. **Build and Test** (10 mins)
   - `anchor build` to compile everything
   - `anchor test` to run integration tests

4. **Deploy to Devnet** (15 mins)
   - Get test SOL from faucet
   - Deploy and test on live network

## Common Issues & Solutions

### Issue 1: "cargo not found"
**Solution**: Restart terminal or run `source $HOME/.cargo/env`

### Issue 2: "anchor: command not found"
**Solution**: Add to PATH: `export PATH="$HOME/.cargo/bin:$PATH"`

### Issue 3: Build fails with "platform-tools" error
**Solution**: Clear cache: `rm -rf ~/.cache/solana`

### Issue 4: WSL performance slow
**Solution**: Move project into WSL filesystem:
```bash
cp -r /mnt/c/Users/cc/claude\ code\ privacy\ solana ~/
cd ~/claude\ code\ privacy\ solana
```

## Verification Checklist

After installation, verify:
- [ ] `solana --version` shows 2.x.x
- [ ] `anchor --version` shows 0.31.x
- [ ] `cargo --version` shows 1.7x.x
- [ ] `rustc --version` shows 1.7x.x
- [ ] `node --version` shows v20.x.x

## Next Steps After Installation

1. **Run Poseidon Test** ← CRITICAL
   ```bash
   cargo test test_poseidon_consistency
   ```

2. **Build Program**
   ```bash
   anchor build
   ```

3. **Deploy to Devnet**
   ```bash
   solana config set --url devnet
   anchor deploy
   ```

4. **Test Deposit/Withdrawal**
   ```bash
   anchor test
   ```

## Summary

**Total Time**: ~1 hour
**Difficulty**: Medium
**Required**: Yes, to compile and deploy

Once installed, we can:
- ✅ Verify Poseidon consistency (finally!)
- ✅ Build the Solana program
- ✅ Deploy to devnet for testing
- ✅ Run full integration tests

**CTO Note**: WSL is essential for Windows users. Native Windows has too many compatibility issues with Solana tooling.