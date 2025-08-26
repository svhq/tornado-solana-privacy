#!/usr/bin/env node

/**
 * Generate a test proof for the Tornado Cash Solana implementation
 * This creates properly formatted proof data that can be used to test
 * the on-chain verification.
 */

const snarkjs = require('snarkjs');
const { buildPoseidon } = require('circomlibjs');
const { PublicKey } = require('@solana/web3.js');

// Convert field element to 32-byte array (big-endian)
function fieldToBytes32(fieldElement) {
    const hex = BigInt(fieldElement).toString(16).padStart(64, '0');
    const bytes = [];
    for (let i = 0; i < 64; i += 2) {
        bytes.push(parseInt(hex.slice(i, i + 2), 16));
    }
    return Buffer.from(bytes);
}

// Split Solana address into high/low parts for circuit
function splitAddress(pubkey) {
    const bytes = pubkey.toBytes();
    
    // High part: [0; 16] + first 16 bytes
    const high = Buffer.alloc(32);
    bytes.slice(0, 16).copy(high, 16);
    
    // Low part: [0; 16] + last 16 bytes
    const low = Buffer.alloc(32);
    bytes.slice(16, 32).copy(low, 16);
    
    return {
        high: '0x' + high.toString('hex'),
        low: '0x' + low.toString('hex')
    };
}

// Format proof for Solana (256 bytes)
function formatProofForSolana(proof) {
    // Extract proof components
    const proofA = [proof.pi_a[0], proof.pi_a[1]];
    const proofB = [[proof.pi_b[0][1], proof.pi_b[0][0]], [proof.pi_b[1][1], proof.pi_b[1][0]]];
    const proofC = [proof.pi_c[0], proof.pi_c[1]];
    
    // Convert to bytes (uncompressed format)
    const proofBytes = Buffer.concat([
        fieldToBytes32(proofA[0]),  // 32 bytes
        fieldToBytes32(proofA[1]),  // 32 bytes (total 64 for A)
        fieldToBytes32(proofB[0][0]), // 32 bytes
        fieldToBytes32(proofB[0][1]), // 32 bytes
        fieldToBytes32(proofB[1][0]), // 32 bytes
        fieldToBytes32(proofB[1][1]), // 32 bytes (total 128 for B)
        fieldToBytes32(proofC[0]),  // 32 bytes
        fieldToBytes32(proofC[1]),  // 32 bytes (total 64 for C)
    ]);
    
    return proofBytes; // 256 bytes total
}

async function generateTestProof() {
    // Initialize Poseidon hasher
    const poseidon = await buildPoseidon();
    
    // Test data
    const recipient = new PublicKey('11111111111111111111111111111112');
    const relayer = new PublicKey('11111111111111111111111111111113');
    const fee = 1000000; // 0.001 SOL
    const refund = 0;
    
    // Mock Merkle root and nullifier
    const root = Buffer.from('1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef', 'hex');
    const nullifierHash = Buffer.from('abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890', 'hex');
    
    // Split addresses
    const recipientSplit = splitAddress(recipient);
    const relayerSplit = splitAddress(relayer);
    
    // Prepare circuit inputs
    const circuitInputs = {
        // Public inputs (must be exactly 8)
        root: '0x' + root.toString('hex'),
        nullifierHash: '0x' + nullifierHash.toString('hex'),
        recipientHigh: recipientSplit.high,
        recipientLow: recipientSplit.low,
        relayerHigh: relayerSplit.high,
        relayerLow: relayerSplit.low,
        fee: fee.toString(),
        refund: refund.toString(),
        
        // Private inputs (mock for testing)
        nullifier: '0x' + Buffer.alloc(32, 1).toString('hex'),
        secret: '0x' + Buffer.alloc(32, 2).toString('hex'),
        pathElements: [], // Merkle path
        pathIndices: [],  // Merkle indices
    };
    
    console.log('Circuit Inputs:');
    console.log('===============');
    console.log('Public Inputs (8):');
    console.log('1. root:', circuitInputs.root);
    console.log('2. nullifierHash:', circuitInputs.nullifierHash);
    console.log('3. recipientHigh:', circuitInputs.recipientHigh);
    console.log('4. recipientLow:', circuitInputs.recipientLow);
    console.log('5. relayerHigh:', circuitInputs.relayerHigh);
    console.log('6. relayerLow:', circuitInputs.relayerLow);
    console.log('7. fee:', circuitInputs.fee);
    console.log('8. refund:', circuitInputs.refund);
    
    // Check if circuit files exist
    const fs = require('fs');
    const wasmPath = 'circuits/withdraw.wasm';
    const zkeyPath = 'circuits/withdraw_final.zkey';
    
    if (!fs.existsSync(wasmPath) || !fs.existsSync(zkeyPath)) {
        console.log('\n⚠️  Circuit files not found!');
        console.log('Please compile your circuit first:');
        console.log('  circom withdraw.circom --r1cs --wasm');
        console.log('  Then run the trusted setup');
        
        // Generate mock proof for testing
        console.log('\n📦 Generating MOCK proof for testing...');
        const mockProof = {
            pi_a: ['1', '2'],
            pi_b: [['3', '4'], ['5', '6']],
            pi_c: ['7', '8']
        };
        
        const proofBytes = formatProofForSolana(mockProof);
        console.log('\nMock Proof (256 bytes):');
        console.log('0x' + proofBytes.toString('hex'));
        
        // Format public inputs for Solana
        const publicInputsForSolana = [
            fieldToBytes32(circuitInputs.root),
            fieldToBytes32(circuitInputs.nullifierHash),
            Buffer.from(recipientSplit.high.slice(2), 'hex'),
            Buffer.from(recipientSplit.low.slice(2), 'hex'),
            Buffer.from(relayerSplit.high.slice(2), 'hex'),
            Buffer.from(relayerSplit.low.slice(2), 'hex'),
            fieldToBytes32(fee),
            fieldToBytes32(refund),
        ];
        
        console.log('\nPublic Inputs for Solana (8 x 32 bytes):');
        publicInputsForSolana.forEach((input, i) => {
            console.log(`${i + 1}. 0x${input.toString('hex')}`);
        });
        
        return;
    }
    
    // Generate real proof
    console.log('\n🔐 Generating real proof...');
    const { proof, publicSignals } = await snarkjs.groth16.fullProve(
        circuitInputs,
        wasmPath,
        zkeyPath
    );
    
    // Verify locally
    const vKey = JSON.parse(fs.readFileSync('circuits/verification_key.json'));
    const verified = await snarkjs.groth16.verify(vKey, publicSignals, proof);
    
    console.log('\n✅ Local verification:', verified ? 'PASSED' : 'FAILED');
    
    // Format for Solana
    const proofBytes = formatProofForSolana(proof);
    console.log('\nProof for Solana (256 bytes):');
    console.log('0x' + proofBytes.toString('hex'));
    
    // Format public signals
    console.log('\nPublic Signals from Circuit:');
    publicSignals.forEach((signal, i) => {
        console.log(`${i + 1}. ${signal}`);
    });
    
    // Save to file
    const output = {
        proof: '0x' + proofBytes.toString('hex'),
        publicInputs: publicSignals,
        recipient: recipient.toBase58(),
        relayer: relayer.toBase58(),
        fee: fee,
        refund: refund,
    };
    
    fs.writeFileSync('test_proof.json', JSON.stringify(output, null, 2));
    console.log('\n💾 Saved to test_proof.json');
}

// Run
generateTestProof().catch(console.error);