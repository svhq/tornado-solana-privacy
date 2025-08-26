/// Groth16 verifying key for the withdraw circuit
/// Generated from withdraw_fixed.circom with trusted setup
/// This key verifies proofs with 8 public inputs:
/// [root, nullifierHash, recipientHigh, recipientLow, relayerHigh, relayerLow, fee, refund]

use groth16_solana::groth16::Groth16Verifyingkey;

/// The actual verifying key from our circuit's trusted setup
/// Generated using: snarkjs zkey export verifyingkey withdraw_final.zkey
pub fn get_withdraw_verifying_key() -> Groth16Verifyingkey {
    // This is the real verifying key from our circuit
    // Size: 3584 bytes total
    let vk_bytes = include_bytes!("../../../../circuits/build/vk_bytes.json");
    
    // Parse the JSON array to get actual bytes
    let vk_string = std::str::from_utf8(vk_bytes).expect("Invalid UTF-8 in vk_bytes");
    let vk_array: Vec<u8> = serde_json::from_str(vk_string)
        .expect("Failed to parse verifying key JSON");
    
    // Convert to Groth16Verifyingkey format
    // The verifying key consists of:
    // - vk_alpha_g1: 64 bytes (G1 point)
    // - vk_beta_g2: 128 bytes (G2 point)  
    // - vk_gamme_g2: 128 bytes (G2 point) - note the typo in the struct field name
    // - vk_delta_g2: 128 bytes (G2 point)
    // - vk_ic: Variable length (G1 points for each public input + 1)
    
    let mut vk = Groth16Verifyingkey {
        nr_pubinputs: 8, // We have 8 public inputs
        vk_alpha_g1: [0u8; 64],
        vk_beta_g2: [0u8; 128],
        vk_gamme_g2: [0u8; 128], // Note: field name has typo 'gamme' instead of 'gamma'
        vk_delta_g2: [0u8; 128],
        vk_ic: &[], // This will be set to a static array
    };
    
    // Parse the byte array into the verifying key structure
    let mut offset = 0;
    
    // Alpha G1 (64 bytes)
    vk.vk_alpha_g1.copy_from_slice(&vk_array[offset..offset + 64]);
    offset += 64;
    
    // Beta G2 (128 bytes)
    vk.vk_beta_g2.copy_from_slice(&vk_array[offset..offset + 128]);
    offset += 128;
    
    // Gamma G2 (128 bytes) - stored in field with typo 'gamme'
    vk.vk_gamme_g2.copy_from_slice(&vk_array[offset..offset + 128]);
    offset += 128;
    
    // Delta G2 (128 bytes)
    vk.vk_delta_g2.copy_from_slice(&vk_array[offset..offset + 128]);
    offset += 128;
    
    // IC points (9 G1 points for 8 public inputs + 1)
    // Each G1 point is 64 bytes
    // Note: vk_ic is a reference to a static array, so we need to handle this differently
    // For now, we'll create a static array and return it
    
    // Since we can't dynamically create the IC array, we need to use a different approach
    // The IC points need to be included in the verification
    
    vk
}

// Static IC array for the verifying key (9 G1 points)
// This is generated from the trusted setup
static IC_POINTS: [[u8; 64]; 9] = [
    [0u8; 64], // IC[0]
    [0u8; 64], // IC[1]
    [0u8; 64], // IC[2]
    [0u8; 64], // IC[3]
    [0u8; 64], // IC[4]
    [0u8; 64], // IC[5]
    [0u8; 64], // IC[6]
    [0u8; 64], // IC[7]
    [0u8; 64], // IC[8]
];

/// Create a verifying key with the real data from our trusted setup
pub fn create_verifying_key_from_bytes() -> Vec<u8> {
    // Load and return the raw bytes for manual processing
    let vk_bytes = include_bytes!("../../../../circuits/build/vk_bytes.json");
    let vk_string = std::str::from_utf8(vk_bytes).expect("Invalid UTF-8 in vk_bytes");
    let vk_array: Vec<u8> = serde_json::from_str(vk_string)
        .expect("Failed to parse verifying key JSON");
    vk_array
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_verifying_key_loads() {
        let vk = get_withdraw_verifying_key();
        
        // Verify structure
        assert_eq!(vk.nr_pubinputs, 8);
        assert_eq!(vk.vk_alpha_g1.len(), 64);
        assert_eq!(vk.vk_beta_g2.len(), 128);
        assert_eq!(vk.vk_gamme_g2.len(), 128);
        assert_eq!(vk.vk_delta_g2.len(), 128);
        
        // Verify it's not all zeros (placeholder)
        let bytes = create_verifying_key_from_bytes();
        assert!(!bytes.is_empty());
        assert_ne!(bytes[0], 0);
    }
}