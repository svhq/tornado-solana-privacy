#[cfg(test)]
mod real_proof_tests {
    use crate::{verify_proof, get_circuit_verifying_key};
    use anchor_lang::prelude::*;
    
    /// Real proof generated from withdraw_fixed.circom
    /// This proof was created with valid inputs and proper trusted setup
    #[test]
    fn test_real_proof_verification() {
        // Real proof from circuits/test_proof_valid.json
        let proof_hex = "1932c68d13e4e1dce10877fb867b64f4eeb14438acb7d96911c00963ae8892fb1100ad50a064e95082e8d9a4fec8729a0b5f661fd118930934e6f78a0fee3c701da6fa818ef65c4d648ae4f871929d51235c7bc5d5f9218745f5cd0bdea50ad327d5f609d882ae5bbe9872c46866b799dd134dc1734b9cfd2db98ae953975b68102a77cbe32a0714b8a82d59ecebcf6a8caf8ff445b5dca2265e7f35eeb6a8062324a790f811da839b12b02cadb62bcc7fe9e713523c4122c8591ca4cd0111a80ce792e8b41714924c86758605f6403297a9030c424f6c1dd48c0abcfa3fd9c6063e61773609fd0338923bcb58bce991192b83a6c3ab299916982e52fea008e3";
        
        // Convert hex string to bytes
        let proof = hex::decode(proof_hex).expect("Invalid proof hex");
        assert_eq!(proof.len(), 256, "Proof must be exactly 256 bytes");
        
        // Real public inputs from the circuit
        let root = hex::decode("2ff370c60cf13d3fffa72d1efe3150948a8c84a664c43d427e25b59a01fe3e3c")
            .expect("Invalid root hex");
        let nullifier_hash = hex::decode("09ca96f9b5a778899e61078e62a5edfe492398e79db303e0440ee2d6e0e4e7f2")
            .expect("Invalid nullifier hex");
        
        // Reconstruct addresses from high/low parts
        // Recipient: 0xe31d835d8657f921fdd87d952db48ec74a949b540a9151fd066c05f7d5c7edd3
        let recipient_bytes = hex::decode("e31d835d8657f921fdd87d952db48ec74a949b540a9151fd066c05f7d5c7edd3")
            .expect("Invalid recipient hex");
        let recipient = Pubkey::new_from_array(recipient_bytes.try_into().unwrap());
        
        // Relayer: 0xc97dda6f4f8d671202378f3843ac899157e5461c0651a0b1cb40541e3397c151
        let relayer_bytes = hex::decode("c97dda6f4f8d671202378f3843ac899157e5461c0651a0b1cb40541e3397c151")
            .expect("Invalid relayer hex");
        let relayer = Pubkey::new_from_array(relayer_bytes.try_into().unwrap());
        
        // Fee and refund
        let fee: u64 = 1_000_000; // 0.001 SOL
        let refund: u64 = 0;
        
        // Get the real verifying key from trusted setup
        let vk = get_circuit_verifying_key();
        
        // Verify the proof
        println!("Testing real proof verification from withdraw circuit...");
        let result = verify_proof(
            &proof,
            &root.try_into().unwrap(),
            &nullifier_hash.try_into().unwrap(),
            &recipient,
            &relayer,
            fee,
            refund,
            &vk,
        );
        
        match result {
            Ok(()) => {
                println!("✅ Real proof verified successfully!");
            }
            Err(e) => {
                println!("❌ Proof verification failed: {:?}", e);
                panic!("Real proof should verify but failed");
            }
        }
    }
    
    /// Test with invalid proof (should fail)
    #[test]
    fn test_invalid_real_proof() {
        // Take the real proof but corrupt it
        let proof_hex = "1932c68d13e4e1dce10877fb867b64f4eeb14438acb7d96911c00963ae8892fb1100ad50a064e95082e8d9a4fec8729a0b5f661fd118930934e6f78a0fee3c701da6fa818ef65c4d648ae4f871929d51235c7bc5d5f9218745f5cd0bdea50ad327d5f609d882ae5bbe9872c46866b799dd134dc1734b9cfd2db98ae953975b68102a77cbe32a0714b8a82d59ecebcf6a8caf8ff445b5dca2265e7f35eeb6a8062324a790f811da839b12b02cadb62bcc7fe9e713523c4122c8591ca4cd0111a80ce792e8b41714924c86758605f6403297a9030c424f6c1dd48c0abcfa3fd9c6063e61773609fd0338923bcb58bce991192b83a6c3ab299916982e52fea008e3";
        
        // Corrupt the proof by changing first byte
        let mut proof = hex::decode(proof_hex).expect("Invalid proof hex");
        proof[0] = proof[0].wrapping_add(1); // Corrupt first byte
        
        // Use same inputs
        let root = hex::decode("2ff370c60cf13d3fffa72d1efe3150948a8c84a664c43d427e25b59a01fe3e3c")
            .unwrap();
        let nullifier_hash = hex::decode("09ca96f9b5a778899e61078e62a5edfe492398e79db303e0440ee2d6e0e4e7f2")
            .unwrap();
        let recipient_bytes = hex::decode("e31d835d8657f921fdd87d952db48ec74a949b540a9151fd066c05f7d5c7edd3")
            .unwrap();
        let recipient = Pubkey::new_from_array(recipient_bytes.try_into().unwrap());
        let relayer_bytes = hex::decode("c97dda6f4f8d671202378f3843ac899157e5461c0651a0b1cb40541e3397c151")
            .unwrap();
        let relayer = Pubkey::new_from_array(relayer_bytes.try_into().unwrap());
        
        let vk = get_circuit_verifying_key();
        
        // This should fail
        let result = verify_proof(
            &proof,
            &root.try_into().unwrap(),
            &nullifier_hash.try_into().unwrap(),
            &recipient,
            &relayer,
            1_000_000,
            0,
            &vk,
        );
        
        assert!(result.is_err(), "Invalid proof should fail verification");
        println!("✅ Invalid proof correctly rejected");
    }
}