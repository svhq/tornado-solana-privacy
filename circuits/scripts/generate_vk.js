#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

console.log('🔄 Converting Verification Key for Solana...\n');

const vkPath = path.join(__dirname, '../build/verification_key.json');
const outputPath = path.join(__dirname, '../build/vk_solana.json');

try {
    // Read the verification key
    const vk = JSON.parse(fs.readFileSync(vkPath, 'utf8'));
    
    // Convert to Solana-compatible format
    // groth16-solana expects specific byte ordering
    const solanaVk = {
        // Alpha G1 point
        alpha_g1: [
            vk.vk_alpha_1[0],
            vk.vk_alpha_1[1]
        ],
        
        // Beta G2 point
        beta_g2: [
            [vk.vk_beta_2[0][0], vk.vk_beta_2[0][1]],
            [vk.vk_beta_2[1][0], vk.vk_beta_2[1][1]]
        ],
        
        // Gamma G2 point
        gamma_g2: [
            [vk.vk_gamma_2[0][0], vk.vk_gamma_2[0][1]],
            [vk.vk_gamma_2[1][0], vk.vk_gamma_2[1][1]]
        ],
        
        // Delta G2 point
        delta_g2: [
            [vk.vk_delta_2[0][0], vk.vk_delta_2[0][1]],
            [vk.vk_delta_2[1][0], vk.vk_delta_2[1][1]]
        ],
        
        // IC (public input commitments)
        ic: vk.IC.map(point => [point[0], point[1]]),
        
        // Protocol metadata
        protocol: "groth16",
        curve: "bn254",
        nPublic: vk.nPublic
    };
    
    // Convert to bytes for Rust integration
    const vkBytes = convertToBytes(solanaVk);
    
    // Save both formats
    fs.writeFileSync(outputPath, JSON.stringify(solanaVk, null, 2));
    fs.writeFileSync(
        path.join(__dirname, '../build/vk_bytes.json'),
        JSON.stringify(vkBytes, null, 2)
    );
    
    console.log('✅ Verification key converted successfully!\n');
    console.log('Generated files:');
    console.log('  - build/vk_solana.json (human-readable format)');
    console.log('  - build/vk_bytes.json (byte array for Rust)');
    console.log('\n📋 To use in Solana program:');
    console.log('1. Copy the byte array from vk_bytes.json');
    console.log('2. Pass it to initialize() function as verifying_key parameter');
    console.log('3. The program will use it for proof verification\n');
    
    // Show key stats
    console.log('📊 Verification Key Stats:');
    console.log(`  - Public inputs: ${vk.nPublic}`);
    console.log(`  - IC points: ${vk.IC.length}`);
    console.log(`  - Total size: ~${JSON.stringify(vkBytes).length / 1024}KB`);
    
} catch (error) {
    console.error('❌ Conversion failed:', error);
    process.exit(1);
}

function convertToBytes(vk) {
    // Convert field elements to byte arrays (big-endian)
    const toBytes = (num) => {
        // Convert string number to hex, then to bytes
        const hex = BigInt(num).toString(16).padStart(64, '0');
        return Array.from(Buffer.from(hex, 'hex'));
    };
    
    const result = {
        alpha_g1: [
            ...toBytes(vk.alpha_g1[0]),
            ...toBytes(vk.alpha_g1[1])
        ],
        beta_g2: [
            ...toBytes(vk.beta_g2[0][0]),
            ...toBytes(vk.beta_g2[0][1]),
            ...toBytes(vk.beta_g2[1][0]),
            ...toBytes(vk.beta_g2[1][1])
        ],
        gamma_g2: [
            ...toBytes(vk.gamma_g2[0][0]),
            ...toBytes(vk.gamma_g2[0][1]),
            ...toBytes(vk.gamma_g2[1][0]),
            ...toBytes(vk.gamma_g2[1][1])
        ],
        delta_g2: [
            ...toBytes(vk.delta_g2[0][0]),
            ...toBytes(vk.delta_g2[0][1]),
            ...toBytes(vk.delta_g2[1][0]),
            ...toBytes(vk.delta_g2[1][1])
        ],
        ic: vk.ic.flatMap(point => [
            ...toBytes(point[0]),
            ...toBytes(point[1])
        ])
    };
    
    // Flatten to single byte array
    return [
        ...result.alpha_g1,
        ...result.beta_g2,
        ...result.gamma_g2,
        ...result.delta_g2,
        ...result.ic
    ];
}