#!/usr/bin/env node

const { exec } = require('child_process');
const fs = require('fs');
const path = require('path');

console.log('🔨 Compiling Tornado Cash Withdraw Circuit...\n');

const circuitPath = path.join(__dirname, '../withdraw_fixed.circom');
const outputDir = path.join(__dirname, '../build');

// Create build directory if it doesn't exist
if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
}

// Compile the circuit using local circom binary
const circomPath = path.join(__dirname, '../circom.exe');
const includePath = path.join(__dirname, '../node_modules');
const compileCommand = `"${circomPath}" "${circuitPath}" --r1cs --wasm --sym --c -o "${outputDir}" -l "${includePath}"`;

console.log('Running:', compileCommand);
console.log('This may take a few minutes...\n');

exec(compileCommand, (error, stdout, stderr) => {
    if (error) {
        console.error('❌ Compilation failed:', error);
        process.exit(1);
    }
    
    if (stderr) {
        console.warn('⚠️ Warnings:', stderr);
    }
    
    console.log(stdout);
    console.log('✅ Circuit compiled successfully!');
    console.log('\nGenerated files:');
    console.log('  - build/withdraw_fixed.r1cs (constraint system)');
    console.log('  - build/withdraw_fixed_js/withdraw_fixed.wasm (witness generator)');
    console.log('  - build/withdraw_fixed.sym (debug symbols)');
    console.log('\nNext step: Run "npm run setup" to generate proving/verifying keys');
});