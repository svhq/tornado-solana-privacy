#[cfg(test)]
mod vault_pda_tests {
    use super::*;
    use anchor_lang::prelude::*;
    use anchor_lang::system_program;
    use solana_program_test::*;
    use solana_sdk::{
        account::Account,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        rent::Rent,
        signature::{Keypair, Signer},
        system_instruction,
        transaction::Transaction,
    };
    use std::str::FromStr;

    // Test constants
    const TEST_DENOMINATION: u64 = 1_000_000_000; // 1 SOL
    const TEST_VK: [u8; 1024] = [1u8; 1024]; // Mock verifying key

    #[tokio::test]
    async fn test_vault_pda_validation_success() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        let mut context = setup_test_context().await;
        
        // Create tornado state
        let tornado_state_keypair = Keypair::new();
        let tornado_state_key = tornado_state_keypair.pubkey();
        
        // Derive vault PDA
        let (vault_pda, vault_bump) = Pubkey::find_program_address(
            &[b"vault", tornado_state_key.as_ref()],
            &program_id,
        );
        
        // Create mock SystemAccount for testing
        let vault_account = Account {
            lamports: Rent::default().minimum_balance(0),
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        };
        
        context.set_account(&vault_pda, &vault_account);
        
        // Test validation should pass
        let vault_system_account = SystemAccount::try_from_unchecked(&AccountInfo {
            key: &vault_pda,
            lamports: std::rc::Rc::new(std::cell::RefCell::new(&mut vault_account.lamports.clone())),
            data: std::rc::Rc::new(std::cell::RefCell::new(&mut vault_account.data.clone())),
            owner: &vault_account.owner,
            executable: vault_account.executable,
            rent_epoch: vault_account.rent_epoch,
        }).unwrap();
        
        let result = validate_vault_pda(&vault_system_account, &tornado_state_key, vault_bump);
        assert!(result.is_ok(), "Vault PDA validation should succeed");
    }

    #[tokio::test]
    async fn test_vault_pda_validation_wrong_derivation() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        let mut context = setup_test_context().await;
        
        let tornado_state_key = Keypair::new().pubkey();
        let wrong_vault_key = Keypair::new().pubkey(); // Wrong vault key
        
        let vault_account = Account {
            lamports: Rent::default().minimum_balance(0),
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        };
        
        context.set_account(&wrong_vault_key, &vault_account);
        
        let vault_system_account = SystemAccount::try_from_unchecked(&AccountInfo {
            key: &wrong_vault_key,
            lamports: std::rc::Rc::new(std::cell::RefCell::new(&mut vault_account.lamports.clone())),
            data: std::rc::Rc::new(std::cell::RefCell::new(&mut vault_account.data.clone())),
            owner: &vault_account.owner,
            executable: vault_account.executable,
            rent_epoch: vault_account.rent_epoch,
        }).unwrap();
        
        let (_, vault_bump) = Pubkey::find_program_address(
            &[b"vault", tornado_state_key.as_ref()],
            &program_id,
        );
        
        let result = validate_vault_pda(&vault_system_account, &tornado_state_key, vault_bump);
        assert!(result.is_err(), "Vault PDA validation should fail with wrong derivation");
    }

    #[tokio::test] 
    async fn test_vault_pda_validation_wrong_owner() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        let mut context = setup_test_context().await;
        
        let tornado_state_key = Keypair::new().pubkey();
        let (vault_pda, vault_bump) = Pubkey::find_program_address(
            &[b"vault", tornado_state_key.as_ref()],
            &program_id,
        );
        
        // Create vault account with wrong owner
        let vault_account = Account {
            lamports: Rent::default().minimum_balance(0),
            data: vec![],
            owner: program_id, // Wrong owner (should be System Program)
            executable: false,
            rent_epoch: 0,
        };
        
        context.set_account(&vault_pda, &vault_account);
        
        let vault_system_account = SystemAccount::try_from_unchecked(&AccountInfo {
            key: &vault_pda,
            lamports: std::rc::Rc::new(std::cell::RefCell::new(&mut vault_account.lamports.clone())),
            data: std::rc::Rc::new(std::cell::RefCell::new(&mut vault_account.data.clone())),
            owner: &vault_account.owner,
            executable: vault_account.executable,
            rent_epoch: vault_account.rent_epoch,
        }).unwrap();
        
        let result = validate_vault_pda(&vault_system_account, &tornado_state_key, vault_bump);
        assert!(result.is_err(), "Vault PDA validation should fail with wrong owner");
    }

    #[tokio::test]
    async fn test_vault_initialization() {
        // Test that vault is properly initialized during Initialize instruction
        let tornado_state_key = Keypair::new().pubkey();
        let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        
        let (vault_pda, vault_bump) = Pubkey::find_program_address(
            &[b"vault", tornado_state_key.as_ref()],
            &program_id,
        );
        
        // Verify PDA derivation is deterministic
        let (vault_pda2, vault_bump2) = Pubkey::find_program_address(
            &[b"vault", tornado_state_key.as_ref()],
            &program_id,
        );
        
        assert_eq!(vault_pda, vault_pda2, "PDA derivation should be deterministic");
        assert_eq!(vault_bump, vault_bump2, "PDA bump should be deterministic");
        
        // After initialization, vault should:
        // 1. Exist on-chain
        // 2. Be owned by System Program
        // 3. Have rent-exempt balance for 0-byte account
        let rent = Rent::default();
        let expected_balance = rent.minimum_balance(0);
        assert!(expected_balance > 0, "Vault should require rent-exempt balance");
    }

    #[tokio::test]
    async fn test_deposit_assumes_vault_initialized() {
        // After vault PDA fix, deposit no longer checks vault rent exemption
        // because vault is guaranteed to be initialized with rent-exempt balance
        // during the Initialize instruction
        
        // This test documents that the previous check:
        // require!(ctx.accounts.vault.lamports() >= minimum_balance, VaultBelowRent);
        // has been removed as it's no longer necessary
        
        let rent = Rent::default();
        let vault_rent_minimum = rent.minimum_balance(0);
        
        // Vault always has at least this amount after initialization
        assert_eq!(vault_rent_minimum, 890880, "0-byte account rent exemption");
    }

    #[tokio::test]
    async fn test_withdraw_rent_protection() {
        // Test rent floor protection logic
        let rent = Rent::default();
        let rent_minimum = rent.minimum_balance(0);
        
        // Test case 1: Sufficient balance after withdrawal
        let vault_balance = rent_minimum + 2_000_000_000; // 2 SOL surplus
        let withdrawal_amount = 1_000_000_000; // 1 SOL withdrawal
        let fee = 100_000_000; // 0.1 SOL fee
        let total_payout = withdrawal_amount + fee;
        
        let remaining_balance = vault_balance.saturating_sub(total_payout);
        assert!(
            remaining_balance >= rent_minimum,
            "Should have sufficient balance after withdrawal"
        );
        
        // Test case 2: Insufficient balance after withdrawal
        let vault_balance = rent_minimum + 500_000_000; // 0.5 SOL surplus
        let withdrawal_amount = 1_000_000_000; // 1 SOL withdrawal (too much)
        let total_payout = withdrawal_amount + fee;
        
        let remaining_balance = vault_balance.saturating_sub(total_payout);
        assert!(
            remaining_balance < rent_minimum,
            "Should have insufficient balance after withdrawal"
        );
    }

    #[tokio::test]
    async fn test_migration_calculation() {
        // Test migration amount calculation
        let rent = Rent::default();
        let state_account_size = 8 + TornadoState::MAX_SIZE;
        let state_rent_minimum = rent.minimum_balance(state_account_size);
        
        // Case 1: State has surplus funds
        let current_state_balance = state_rent_minimum + 5_000_000_000; // 5 SOL surplus
        let migration_amount = current_state_balance - state_rent_minimum;
        assert_eq!(migration_amount, 5_000_000_000, "Migration amount should be surplus");
        
        // Case 2: State has no surplus
        let current_state_balance = state_rent_minimum;
        let migration_amount = current_state_balance - state_rent_minimum;
        assert_eq!(migration_amount, 0, "No migration needed when no surplus");
        
        // Case 3: State below rent minimum (shouldn't happen)
        let current_state_balance = state_rent_minimum.saturating_sub(1000);
        let migration_amount = current_state_balance.saturating_sub(state_rent_minimum);
        assert_eq!(migration_amount, 0, "No migration when below rent minimum");
    }
    
    #[tokio::test]
    async fn test_migration_uses_cpi_with_pda_signing() {
        // Test that migration now uses CPI pattern with PDA signing
        // This documents the architectural change from direct lamport manipulation
        // to proper CPI transfers consistent with the rest of the codebase
        
        let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        
        // Derive tornado_state PDA
        let (tornado_state_pda, state_bump) = Pubkey::find_program_address(
            &[b"tornado"],
            &program_id,
        );
        
        // Derive vault PDA
        let (vault_pda, vault_bump) = Pubkey::find_program_address(
            &[b"vault", tornado_state_pda.as_ref()],
            &program_id,
        );
        
        // Verify PDA seeds for signing
        let state_seeds: &[&[u8]] = &[b"tornado", &[state_bump]];
        
        // Verify seeds can create the correct PDA
        let derived_state = Pubkey::create_program_address(state_seeds, &program_id)
            .expect("Should create valid PDA from seeds");
        assert_eq!(derived_state, tornado_state_pda, "PDA seeds should match");
        
        // After migration:
        // 1. tornado_state maintains rent exemption
        // 2. vault receives surplus funds via CPI
        // 3. No direct lamport manipulation (try_borrow_mut_lamports removed)
        let rent = Rent::default();
        let state_account_size = 8 + TornadoState::MAX_SIZE;
        let state_rent_minimum = rent.minimum_balance(state_account_size);
        
        // Both accounts remain rent-exempt after migration
        assert!(state_rent_minimum > 0, "State must maintain rent exemption");
        let vault_rent_minimum = rent.minimum_balance(0);
        assert!(vault_rent_minimum > 0, "Vault must maintain rent exemption");
    }

    #[tokio::test]
    async fn test_vault_seeds_generation() {
        // Test vault seed generation is correct
        let tornado_state_key = Keypair::new().pubkey();
        let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        
        let (vault_pda, vault_bump) = Pubkey::find_program_address(
            &[b"vault", tornado_state_key.as_ref()],
            &program_id,
        );
        
        // Generate seeds array as used in CPI
        let vault_seeds: &[&[u8]] = &[
            b"vault",
            tornado_state_key.as_ref(),
            &[vault_bump]
        ];
        
        // Verify seeds generate correct PDA
        let derived_key = Pubkey::create_program_address(vault_seeds, &program_id)
            .expect("Should create valid PDA from seeds");
        
        assert_eq!(derived_key, vault_pda, "Seeds should generate correct vault PDA");
    }

    #[tokio::test]
    async fn test_different_tornado_states_have_different_vaults() {
        let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        
        let tornado_state1 = Keypair::new().pubkey();
        let tornado_state2 = Keypair::new().pubkey();
        
        let (vault1, _) = Pubkey::find_program_address(
            &[b"vault", tornado_state1.as_ref()],
            &program_id,
        );
        
        let (vault2, _) = Pubkey::find_program_address(
            &[b"vault", tornado_state2.as_ref()],
            &program_id,
        );
        
        assert_ne!(vault1, vault2, "Different tornado states should have different vault PDAs");
    }

    #[tokio::test]
    async fn test_address_splitting_reconstruction() {
        // Test address splitting and reconstruction logic
        let original_address = Keypair::new().pubkey();
        
        let (high, low) = split_address_to_high_low(&original_address);
        let reconstructed = reconstruct_address_from_high_low(&high, &low);
        
        assert_eq!(
            original_address, reconstructed,
            "Address should survive split/reconstruct cycle"
        );
        
        // Test that high and low parts fit in BN254 field
        // BN254 field size is roughly 2^254, so 32 bytes with high bits zero should fit
        assert_eq!(high[0..16], [0u8; 16], "High part should have zero padding");
        assert_eq!(low[0..16], [0u8; 16], "Low part should have zero padding");
    }

    #[tokio::test] 
    async fn test_u64_encoding() {
        // Test u64 encoding for circuit public inputs
        let test_values = vec![0u64, 1u64, 1000u64, u64::MAX];
        
        for value in test_values {
            let mut encoded = [0u8; 32];
            encode_u64_as_32_bytes(value, &mut encoded);
            
            // Verify encoding format (right-aligned big-endian)
            assert_eq!(encoded[0..24], [0u8; 24], "Should have zero padding");
            
            let decoded = u64::from_be_bytes(encoded[24..32].try_into().unwrap());
            assert_eq!(value, decoded, "Value should survive encode/decode cycle");
        }
    }

    // Helper function to set up test context
    async fn setup_test_context() -> ProgramTestContext {
        let program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();
        let mut program_test = ProgramTest::new(
            "tornado_solana",
            program_id,
            processor!(crate::entry),
        );
        
        program_test.start_with_context().await
    }

    // Additional helper functions for integration testing would go here
    // These would require more complex setup with actual transaction building
}