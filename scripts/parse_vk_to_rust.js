#!/usr/bin/env node

/**
 * Converts a snarkjs verification_key.json to Rust code for groth16-solana
 * 
 * Usage: node parse_vk_to_rust.js verification_key.json > vk.rs
 * 
 * This script handles the conversion from snarkjs format to the format
 * expected by groth16-solana's Groth16Verifyingkey struct.
 */

const fs = require('fs');

function parseG1Point(point) {
    // G1 points are [x, y] in snarkjs, need to be 64 bytes (32 + 32)
    const x = BigInt(point[0]);
    const y = BigInt(point[1]);
    
    const xBytes = x.toString(16).padStart(64, '0');
    const yBytes = y.toString(16).padStart(64, '0');
    
    // Convert to byte array format
    const bytes = [];
    for (let i = 0; i < 64; i += 2) {
        bytes.push(`0x${xBytes.slice(i, i + 2)}`);
    }
    for (let i = 0; i < 64; i += 2) {
        bytes.push(`0x${yBytes.slice(i, i + 2)}`);
    }
    
    return bytes;
}

function parseG2Point(point) {
    // G2 points are [[x0, x1], [y0, y1]] in snarkjs
    const x0 = BigInt(point[0][0]);
    const x1 = BigInt(point[0][1]);
    const y0 = BigInt(point[1][0]);
    const y1 = BigInt(point[1][1]);
    
    const x0Bytes = x0.toString(16).padStart(64, '0');
    const x1Bytes = x1.toString(16).padStart(64, '0');
    const y0Bytes = y0.toString(16).padStart(64, '0');
    const y1Bytes = y1.toString(16).padStart(64, '0');
    
    // Convert to byte array format (128 bytes total)
    const bytes = [];
    for (const hex of [x0Bytes, x1Bytes, y0Bytes, y1Bytes]) {
        for (let i = 0; i < 64; i += 2) {
            bytes.push(`0x${hex.slice(i, i + 2)}`);
        }
    }
    
    return bytes;
}

function formatByteArray(bytes, indent = '    ') {
    const lines = [];
    for (let i = 0; i < bytes.length; i += 16) {
        const chunk = bytes.slice(i, Math.min(i + 16, bytes.length));
        lines.push(`${indent}${chunk.join(', ')},`);
    }
    return lines.join('\n');
}

function convertVK(vkPath) {
    const vk = JSON.parse(fs.readFileSync(vkPath, 'utf8'));
    
    // Parse all components
    const alphaG1 = parseG1Point(vk.vk_alpha_1);
    const betaG2 = parseG2Point(vk.vk_beta_2);
    const gammaG2 = parseG2Point(vk.vk_gamma_2);
    const deltaG2 = parseG2Point(vk.vk_delta_2);
    
    // Parse IC points (one for each public input + 1)
    const icPoints = vk.IC.map(point => parseG1Point(point));
    
    // Generate Rust code
    let rustCode = `use groth16_solana::groth16::Groth16Verifyingkey;

/// Verifying key for the Tornado Cash withdrawal circuit
/// Generated from trusted setup ceremony
pub const WITHDRAWAL_VERIFYING_KEY: Groth16Verifyingkey = Groth16Verifyingkey {
    nr_pubinputs: ${icPoints.length - 1},
    
    vk_alpha_g1: [
${formatByteArray(alphaG1, '        ')}
    ],
    
    vk_beta_g2: [
${formatByteArray(betaG2, '        ')}
    ],
    
    vk_gamme_g2: [ // Note: typo in original struct name
${formatByteArray(gammaG2, '        ')}
    ],
    
    vk_delta_g2: [
${formatByteArray(deltaG2, '        ')}
    ],
    
    vk_ic: &[`;

    // Add each IC point
    icPoints.forEach((point, index) => {
        rustCode += `
        // IC[${index}]
        [
${formatByteArray(point, '            ')}
        ],`;
    });
    
    rustCode += `
    ],
};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vk_has_correct_public_inputs() {
        assert_eq!(WITHDRAWAL_VERIFYING_KEY.nr_pubinputs, 8);
        assert_eq!(WITHDRAWAL_VERIFYING_KEY.vk_ic.len(), 9); // 8 + 1
    }
    
    #[test]
    fn test_vk_point_sizes() {
        assert_eq!(WITHDRAWAL_VERIFYING_KEY.vk_alpha_g1.len(), 64);
        assert_eq!(WITHDRAWAL_VERIFYING_KEY.vk_beta_g2.len(), 128);
        assert_eq!(WITHDRAWAL_VERIFYING_KEY.vk_gamme_g2.len(), 128);
        assert_eq!(WITHDRAWAL_VERIFYING_KEY.vk_delta_g2.len(), 128);
    }
}
`;

    return rustCode;
}

// Main execution
if (process.argv.length !== 3) {
    console.error('Usage: node parse_vk_to_rust.js <verification_key.json>');
    process.exit(1);
}

try {
    const rustCode = convertVK(process.argv[2]);
    console.log(rustCode);
} catch (error) {
    console.error('Error:', error.message);
    process.exit(1);
}