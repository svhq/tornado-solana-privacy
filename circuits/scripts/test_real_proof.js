#!/usr/bin/env node

const snarkjs = require('snarkjs');
const { buildPoseidon } = require('circomlibjs');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

console.log('🔬 Testing Tornado Cash Circuit with Real Proof Generation\n');

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

// Function to generate mock merkle proof
function generateMockMerkleProof(levels) {
    const pathElements = [];
    const pathIndices = [];
    
    for (let i = 0; i < levels; i++) {
        // Generate random 32-byte path elements
        pathElements.push(BigInt('0x' + crypto.randomBytes(31).toString('hex')).toString());
        pathIndices.push(Math.floor(Math.random() * 2));
    }
    
    return { pathElements, pathIndices };
}

async function generateRealProof() {
    try {
        console.log('🔐 Initializing Poseidon hasher...');
        const poseidon = await buildPoseidon();
        
        console.log('📋 Generating test inputs...\n');
        
        // Generate random secrets
        const nullifier = BigInt('0x' + crypto.randomBytes(31).toString('hex'));
        const secret = BigInt('0x' + crypto.randomBytes(31).toString('hex'));
        
        console.log('Private Inputs:');
        console.log('- nullifier:', '0x' + nullifier.toString(16).padStart(64, '0'));
        console.log('- secret:', '0x' + secret.toString(16).padStart(64, '0'));
        
        // Compute commitment using Poseidon
        const commitmentField = poseidon([nullifier, secret]);
        console.log('- commitment:', '0x' + poseidon.F.toString(commitmentField, 16).padStart(64, '0'));
        
        // Compute nullifier hash using Poseidon
        const nullifierHashField = poseidon([nullifier]);
        console.log('- nullifierHash:', '0x' + poseidon.F.toString(nullifierHashField, 16).padStart(64, '0'));
        
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
        
        // Generate mock merkle proof
        const merkleProof = generateMockMerkleProof(20);
        const mockRoot = BigInt('0x' + crypto.randomBytes(31).toString('hex'));
        
        console.log('\nMerkle Tree:');
        console.log('- root:', '0x' + mockRoot.toString(16).padStart(64, '0'));
        console.log('- levels:', merkleProof.pathElements.length);
        
        // Transaction parameters
        const fee = 1000000; // 0.001 SOL
        const refund = 0;
        
        console.log('\nTransaction Parameters:');
        console.log('- fee:', fee, 'lamports (0.001 SOL)');
        console.log('- refund:', refund, 'lamports');
        
        // Prepare circuit inputs
        const circuitInputs = {
            // Public inputs (8 total for withdraw_fixed circuit)
            root: mockRoot.toString(),
            nullifierHash: poseidon.F.toString(nullifierHashField),
            recipientHigh: recipientSplit.high,
            recipientLow: recipientSplit.low,
            relayerHigh: relayerSplit.high,
            relayerLow: relayerSplit.low,
            fee: fee.toString(),
            refund: refund.toString(),
            
            // Private inputs
            nullifier: nullifier.toString(),
            secret: secret.toString(),
            pathElements: merkleProof.pathElements,
            pathIndices: merkleProof.pathIndices
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
            console.log(`${i + 1}. ${signal}`);
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
            console.log(`${i + 1}. 0x${fieldToBytes32(signal)}`);
        });
        
        // Save test data
        const testData = {
            proof: '0x' + proofBytes,
            publicInputs: publicSignals.map(s => '0x' + fieldToBytes32(s)),
            metadata: {
                recipientAddress: '0x' + recipientBytes.toString('hex'),
                relayerAddress: '0x' + relayerBytes.toString('hex'),
                fee: fee,
                refund: refund,
                proofGenerationTime: proofTime,
                verificationTime: verifyTime,
                timestamp: new Date().toISOString()
            }
        };
        
        const testDataPath = path.join(__dirname, '../test_proof_real.json');
        fs.writeFileSync(testDataPath, JSON.stringify(testData, null, 2));
        
        console.log(`\n💾 Test data saved to: ${testDataPath}`);
        
        console.log('\n✨ Summary:');
        console.log(`- Circuit: withdraw_fixed.circom (Solana address compatible)`);
        console.log(`- Constraints: 5,897 non-linear, 5,965 linear`);
        console.log(`- Public inputs: 8 (root, nullifierHash, recipientHigh/Low, relayerHigh/Low, fee, refund)`);
        console.log(`- Proof generation: ${proofTime}ms`);
        console.log(`- Verification: ${verifyTime}ms`);
        console.log(`- Ready for Solana deployment! 🎉`);
        
    } catch (error) {
        console.error('❌ Test failed:', error);
        
        if (error.message.includes('Error in template MerkleTreeChecker')) {
            console.log('\n💡 This might be due to the mock merkle proof not matching the commitment.');
            console.log('In a real scenario, the commitment would be in the merkle tree.');
        }
    }
}

// Run the test
generateRealProof().catch(console.error);