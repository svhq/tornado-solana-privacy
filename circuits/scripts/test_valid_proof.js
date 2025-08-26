#!/usr/bin/env node

const snarkjs = require('snarkjs');
const { buildPoseidon } = require('circomlibjs');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

console.log('🔬 Testing Tornado Cash Circuit with Valid Merkle Proof\n');

// Function to convert address to high/low parts for circuit
function splitSolanaAddress(addressBytes) {
    // Split 32-byte address into two 16-byte parts
    const high = addressBytes.slice(0, 16);
    const low = addressBytes.slice(16, 32);
    
    // Convert to field elements (big integers)
    const highBigInt = BigInt('0x' + Buffer.from(high).toString('hex'));
    const lowBigInt = BigInt('0x' + Buffer.from(low).toString('hex'));
    
    return {
        high: highBigInt.toString(),
        low: lowBigInt.toString()
    };
}

// Function to build a valid merkle tree with the commitment
function buildValidMerkleTree(poseidon, commitment, levels = 20) {
    // Start with our commitment as a leaf
    let currentHash = commitment;
    const pathElements = [];
    const pathIndices = [];
    
    // Build the tree from leaf to root
    for (let i = 0; i < levels; i++) {
        // Generate a random sibling
        const sibling = poseidon.F.random();
        pathElements.push(poseidon.F.toString(sibling));
        
        // Randomly choose left (0) or right (1)
        const isRight = Math.random() < 0.5 ? 1 : 0;
        pathIndices.push(isRight);
        
        // Hash with sibling according to path index
        if (isRight === 0) {
            // current is left, sibling is right
            currentHash = poseidon([currentHash, sibling]);
        } else {
            // current is right, sibling is left  
            currentHash = poseidon([sibling, currentHash]);
        }
    }
    
    return {
        root: currentHash,
        pathElements,
        pathIndices
    };
}

async function generateValidProof() {
    try {
        console.log('🔐 Initializing Poseidon hasher...');
        const poseidon = await buildPoseidon();
        
        console.log('📋 Generating test inputs...\n');
        
        // Generate random secrets
        const nullifier = poseidon.F.random();
        const secret = poseidon.F.random();
        
        console.log('Private Inputs:');
        console.log('- nullifier:', '0x' + poseidon.F.toString(nullifier, 16).padStart(64, '0'));
        console.log('- secret:', '0x' + poseidon.F.toString(secret, 16).padStart(64, '0'));
        
        // Compute commitment using Poseidon
        const commitment = poseidon([nullifier, secret]);
        console.log('- commitment:', '0x' + poseidon.F.toString(commitment, 16).padStart(64, '0'));
        
        // Compute nullifier hash using Poseidon
        const nullifierHash = poseidon([nullifier]);
        console.log('- nullifierHash:', '0x' + poseidon.F.toString(nullifierHash, 16).padStart(64, '0'));
        
        // Build a valid merkle tree with our commitment
        console.log('\n🌲 Building valid merkle tree...');
        const merkleTree = buildValidMerkleTree(poseidon, commitment, 20);
        
        console.log('Merkle Tree:');
        console.log('- root:', '0x' + poseidon.F.toString(merkleTree.root, 16).padStart(64, '0'));
        console.log('- levels: 20');
        console.log('- commitment is leaf at position (random)');
        
        // Generate mock Solana addresses (32 bytes each)
        const recipientBytes = crypto.randomBytes(32);
        const relayerBytes = crypto.randomBytes(32);
        
        // Split addresses into high/low parts
        const recipientSplit = splitSolanaAddress(recipientBytes);
        const relayerSplit = splitSolanaAddress(relayerBytes);
        
        console.log('\nSolana Addresses:');
        console.log('- recipient:', recipientBytes.toString('hex'));
        console.log('  - high:', recipientSplit.high);
        console.log('  - low:', recipientSplit.low);
        console.log('- relayer:', relayerBytes.toString('hex'));
        console.log('  - high:', relayerSplit.high);
        console.log('  - low:', relayerSplit.low);
        
        // Transaction parameters
        const fee = 1000000; // 0.001 SOL
        const refund = 0;
        
        console.log('\nTransaction Parameters:');
        console.log('- fee:', fee, 'lamports (0.001 SOL)');
        console.log('- refund:', refund, 'lamports');
        
        // Prepare circuit inputs
        const circuitInputs = {
            // Public inputs (8 total for withdraw_fixed circuit)
            root: poseidon.F.toString(merkleTree.root),
            nullifierHash: poseidon.F.toString(nullifierHash),
            recipientHigh: recipientSplit.high,
            recipientLow: recipientSplit.low,
            relayerHigh: relayerSplit.high,
            relayerLow: relayerSplit.low,
            fee: fee.toString(),
            refund: refund.toString(),
            
            // Private inputs
            nullifier: poseidon.F.toString(nullifier),
            secret: poseidon.F.toString(secret),
            pathElements: merkleTree.pathElements,
            pathIndices: merkleTree.pathIndices
        };
        
        console.log('\n🔧 Circuit Input Summary:');
        console.log('- Public inputs: 8');
        console.log('- Private inputs: 42 (nullifier, secret, 20 path elements, 20 path indices)');
        console.log('- Total constraints: ~5,897');
        
        // Check if circuit files exist
        const wasmPath = path.join(__dirname, '../build/withdraw_fixed_js/withdraw_fixed.wasm');
        const zkeyPath = path.join(__dirname, '../build/withdraw_final.zkey');
        
        if (!fs.existsSync(wasmPath)) {
            console.error('\n❌ WASM file not found:', wasmPath);
            console.log('Run: npm run compile');
            return;
        }
        
        if (!fs.existsSync(zkeyPath)) {
            console.error('\n❌ Proving key not found:', zkeyPath);
            console.log('Run: npm run setup');
            return;
        }
        
        console.log('\n🔐 Generating zero-knowledge proof...');
        console.log('This may take 10-30 seconds...\n');
        
        const startTime = Date.now();
        
        // Generate the proof
        const { proof, publicSignals } = await snarkjs.groth16.fullProve(
            circuitInputs,
            wasmPath,
            zkeyPath
        );
        
        const proofTime = Date.now() - startTime;
        console.log(`✅ Proof generated in ${proofTime}ms\n`);
        
        // Verify the proof
        console.log('🔍 Verifying proof...');
        const vKeyPath = path.join(__dirname, '../build/verification_key.json');
        const vKey = JSON.parse(fs.readFileSync(vKeyPath, 'utf8'));
        
        const verifyStartTime = Date.now();
        const isValid = await snarkjs.groth16.verify(vKey, publicSignals, proof);
        const verifyTime = Date.now() - verifyStartTime;
        
        if (isValid) {
            console.log(`✅ Proof verified successfully in ${verifyTime}ms!\n`);
        } else {
            console.error('❌ Proof verification failed!\n');
            return;
        }
        
        // Display proof components
        console.log('📦 Proof Components:');
        console.log('- Proof A:', proof.pi_a.slice(0, 2));
        console.log('- Proof B (C0):', proof.pi_b[0]);
        console.log('- Proof B (C1):', proof.pi_b[1]);
        console.log('- Proof C:', proof.pi_c.slice(0, 2));
        
        console.log('\n📋 Public Signals (8):');
        publicSignals.forEach((signal, i) => {
            const labels = ['root', 'nullifierHash', 'recipientHigh', 'recipientLow', 'relayerHigh', 'relayerLow', 'fee', 'refund'];
            console.log(`${i + 1}. ${labels[i]}: ${signal}`);
        });
        
        // Format for Solana
        console.log('\n🚀 Solana Integration Format:');
        
        // Convert proof to bytes (256 bytes total for Groth16)
        function fieldToBytes32(field) {
            const hex = BigInt(field).toString(16).padStart(64, '0');
            return hex;
        }
        
        const proofBytes = [
            fieldToBytes32(proof.pi_a[0]),  // 32 bytes
            fieldToBytes32(proof.pi_a[1]),  // 32 bytes
            fieldToBytes32(proof.pi_b[0][1]), // 32 bytes (note: order swap for Groth16)
            fieldToBytes32(proof.pi_b[0][0]), // 32 bytes
            fieldToBytes32(proof.pi_b[1][1]), // 32 bytes
            fieldToBytes32(proof.pi_b[1][0]), // 32 bytes
            fieldToBytes32(proof.pi_c[0]),  // 32 bytes
            fieldToBytes32(proof.pi_c[1]),  // 32 bytes
        ].join('');
        
        console.log('Proof (256 bytes):');
        console.log('0x' + proofBytes);
        
        console.log('\nPublic Inputs (8 × 32 bytes):');
        publicSignals.forEach((signal, i) => {
            const labels = ['root', 'nullifierHash', 'recipientHigh', 'recipientLow', 'relayerHigh', 'relayerLow', 'fee', 'refund'];
            console.log(`${i + 1}. ${labels[i]}: 0x${fieldToBytes32(signal)}`);
        });
        
        // Save test data
        const testData = {
            proof: '0x' + proofBytes,
            publicInputs: publicSignals.map(s => '0x' + fieldToBytes32(s)),
            publicInputsLabeled: {
                root: '0x' + fieldToBytes32(publicSignals[0]),
                nullifierHash: '0x' + fieldToBytes32(publicSignals[1]),
                recipientHigh: '0x' + fieldToBytes32(publicSignals[2]),
                recipientLow: '0x' + fieldToBytes32(publicSignals[3]),
                relayerHigh: '0x' + fieldToBytes32(publicSignals[4]),
                relayerLow: '0x' + fieldToBytes32(publicSignals[5]),
                fee: publicSignals[6],
                refund: publicSignals[7]
            },
            metadata: {
                recipientAddress: '0x' + recipientBytes.toString('hex'),
                relayerAddress: '0x' + relayerBytes.toString('hex'),
                fee: fee,
                refund: refund,
                proofGenerationTime: proofTime,
                verificationTime: verifyTime,
                timestamp: new Date().toISOString(),
                circuit: 'withdraw_fixed.circom',
                constraints: {
                    nonLinear: 5897,
                    linear: 5965,
                    public: 8,
                    private: 42
                }
            }
        };
        
        const testDataPath = path.join(__dirname, '../test_proof_valid.json');
        fs.writeFileSync(testDataPath, JSON.stringify(testData, null, 2));
        
        console.log(`\n💾 Valid test data saved to: ${testDataPath}`);
        
        console.log('\n🎉 SUCCESS! Real Proof Generated and Verified!');
        console.log('\n✨ Summary:');
        console.log(`- Circuit: withdraw_fixed.circom (Solana address compatible)`);
        console.log(`- Constraints: 5,897 non-linear, 5,965 linear`);
        console.log(`- Public inputs: 8 (root, nullifierHash, recipientHigh/Low, relayerHigh/Low, fee, refund)`);
        console.log(`- Proof generation: ${proofTime}ms`);
        console.log(`- Verification: ${verifyTime}ms`);
        console.log(`- Status: ✅ READY FOR SOLANA DEPLOYMENT`);
        
        // Verification key information
        const vkSolanaPath = path.join(__dirname, '../build/vk_solana.json');
        const vkBytesPath = path.join(__dirname, '../build/vk_bytes.json');
        
        console.log('\n📋 Integration Files:');
        console.log(`- Verification key (Solana): ${vkSolanaPath}`);
        console.log(`- Verification key (bytes): ${vkBytesPath}`);
        console.log(`- Test proof data: ${testDataPath}`);
        
        console.log('\n🚀 Next Steps for Solana Integration:');
        console.log('1. Copy verification key bytes to your Solana program');
        console.log('2. Use the proof and public inputs from test_proof_valid.json');
        console.log('3. Test on-chain verification with groth16-solana');
        console.log('4. Deploy and enjoy private transactions! 🎉');
        
    } catch (error) {
        console.error('❌ Test failed:', error);
    }
}

// Run the test
generateValidProof().catch(console.error);