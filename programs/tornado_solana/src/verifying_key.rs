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
    // - alpha_g1: 64 bytes (G1 point)
    // - beta_g2: 128 bytes (G2 point)  
    // - gamma_g2: 128 bytes (G2 point)
    // - delta_g2: 128 bytes (G2 point)
    // - ic: Variable length (G1 points for each public input + 1)
    
    let mut vk = Groth16Verifyingkey {
        alpha_g1: [0u8; 64],
        beta_g2: [0u8; 128],
        gamma_g2: [0u8; 128],
        delta_g2: [0u8; 128],
        ic: vec![],
    };
    
    // Parse the byte array into the verifying key structure
    let mut offset = 0;
    
    // Alpha G1 (64 bytes)
    vk.alpha_g1.copy_from_slice(&vk_array[offset..offset + 64]);
    offset += 64;
    
    // Beta G2 (128 bytes)
    vk.beta_g2.copy_from_slice(&vk_array[offset..offset + 128]);
    offset += 128;
    
    // Gamma G2 (128 bytes)
    vk.gamma_g2.copy_from_slice(&vk_array[offset..offset + 128]);
    offset += 128;
    
    // Delta G2 (128 bytes)
    vk.delta_g2.copy_from_slice(&vk_array[offset..offset + 128]);
    offset += 128;
    
    // IC points (9 G1 points for 8 public inputs + 1)
    // Each G1 point is 64 bytes
    for _ in 0..9 {
        let mut ic_point = [0u8; 64];
        ic_point.copy_from_slice(&vk_array[offset..offset + 64]);
        vk.ic.push(ic_point);
        offset += 64;
    }
    
    vk
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_verifying_key_loads() {
        let vk = get_withdraw_verifying_key();
        
        // Verify structure
        assert_eq!(vk.alpha_g1.len(), 64);
        assert_eq!(vk.beta_g2.len(), 128);
        assert_eq!(vk.gamma_g2.len(), 128);
        assert_eq!(vk.delta_g2.len(), 128);
        assert_eq!(vk.ic.len(), 9); // 8 public inputs + 1
        
        // Verify it's not all zeros (placeholder)
        assert_ne!(vk.alpha_g1[0], 0);
        assert_ne!(vk.beta_g2[0], 0);
    }
}