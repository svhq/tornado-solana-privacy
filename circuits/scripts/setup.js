#!/usr/bin/env node

const snarkjs = require('snarkjs');
const fs = require('fs');
const path = require('path');

async function runTrustedSetup() {
    console.log('🔐 Running Trusted Setup for Tornado Cash Circuit...\n');
    console.log('⚠️  WARNING: This is for DEVELOPMENT ONLY!');
    console.log('⚠️  For production, use a proper ceremony with multiple contributors\n');

    const r1csPath = path.join(__dirname, '../build/withdraw_fixed.r1cs');
    const ptauPath = path.join(__dirname, '../build/powersOfTau28_hez_final_15.ptau');
    const zkeyPath = path.join(__dirname, '../build/withdraw_0000.zkey');
    const finalZkeyPath = path.join(__dirname, '../build/withdraw_final.zkey');
    const vkeyPath = path.join(__dirname, '../build/verification_key.json');

    try {
        // Step 1: Download Powers of Tau (if not exists)
        if (!fs.existsSync(ptauPath)) {
            console.log('📥 Downloading Powers of Tau file...');
            const https = require('https');
            const file = fs.createWriteStream(ptauPath);
            
            await new Promise((resolve, reject) => {
                https.get('https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_15.ptau', (response) => {
                    response.pipe(file);
                    file.on('finish', () => {
                        file.close();
                        console.log('✅ Powers of Tau downloaded\n');
                        resolve();
                    });
                }).on('error', reject);
            });
        }

        // Step 2: Generate initial zkey
        console.log('🔨 Generating initial proving key...');
        await snarkjs.zKey.newZKey(r1csPath, ptauPath, zkeyPath);
        console.log('✅ Initial zkey generated\n');

        // Step 3: Contribute randomness (ceremony phase 2)
        console.log('🎲 Contributing randomness...');
        await snarkjs.zKey.contribute(zkeyPath, finalZkeyPath, "Tornado Solana Dev", "dev entropy");
        console.log('✅ Randomness contributed\n');

        // Step 4: Export verification key
        console.log('📤 Exporting verification key...');
        const vKey = await snarkjs.zKey.exportVerificationKey(finalZkeyPath);
        fs.writeFileSync(vkeyPath, JSON.stringify(vKey, null, 2));
        console.log('✅ Verification key exported\n');

        // Step 5: Verify the setup
        console.log('🔍 Verifying trusted setup...');
        const isValid = await snarkjs.zKey.verifyFromR1cs(r1csPath, ptauPath, finalZkeyPath);
        
        if (isValid) {
            console.log('✅ Trusted setup verified successfully!\n');
            console.log('Generated files:');
            console.log('  - build/withdraw_final.zkey (proving key)');
            console.log('  - build/verification_key.json (for Solana contract)');
            console.log('\nNext step: Run "npm run generate-vk" to format for Solana');
        } else {
            throw new Error('Trusted setup verification failed');
        }

    } catch (error) {
        console.error('❌ Trusted setup failed:', error);
        process.exit(1);
    }
}

runTrustedSetup();