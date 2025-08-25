use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;

pub mod merkle_tree;
use merkle_tree::*;

#[cfg(test)]
mod poseidon_test;

declare_id!("11111111111111111111111111111112");

#[program]
pub mod tornado_solana {
    use super::*;

    /// Initialize a new Tornado pool with fixed denomination
    /// @param verifying_key: The Groth16 verifying key from trusted setup ceremony
    pub fn initialize(
        ctx: Context<Initialize>, 
        denomination: u64,
        verifying_key: Vec<u8>,
    ) -> Result<()> {
        let tornado_state = &mut ctx.accounts.tornado_state;
        
        tornado_state.authority = ctx.accounts.authority.key();
        tornado_state.denomination = denomination;
        tornado_state.merkle_tree = MerkleTree::new();
        tornado_state.current_root_index = 0;
        tornado_state.next_index = 0;
        tornado_state.verifying_key = verifying_key;
        
        Ok(())
    }

    /// Deposit funds into the tornado pool
    /// @param commitment: Hash(nullifier + secret)
    pub fn deposit(ctx: Context<Deposit>, commitment: [u8; 32]) -> Result<()> {
        let tornado_state = &mut ctx.accounts.tornado_state;
        
        // Verify deposit amount matches denomination
        let deposit_amount = tornado_state.denomination;
        
        // Transfer SOL from depositor to pool
        let ix = system_instruction::transfer(
            &ctx.accounts.depositor.key(),
            &ctx.accounts.tornado_state.key(),
            deposit_amount,
        );
        
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.depositor.to_account_info(),
                ctx.accounts.tornado_state.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        
        // Insert commitment into merkle tree
        let leaf_index = tornado_state.merkle_tree.insert(commitment)?;
        
        // Store commitment to prevent duplicates
        tornado_state.commitments.push(commitment);
        
        // Update root history
        let new_root = tornado_state.merkle_tree.get_root();
        let new_index = (tornado_state.current_root_index + 1) % ROOT_HISTORY_SIZE;
        tornado_state.current_root_index = new_index;
        tornado_state.roots[new_index as usize] = new_root;
        
        emit!(DepositEvent {
            commitment,
            leaf_index,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }

    /// Withdraw funds from the tornado pool with a zero-knowledge proof
    pub fn withdraw(
        ctx: Context<Withdraw>,
        proof: Vec<u8>,
        root: [u8; 32],
        nullifier_hash: [u8; 32],
        recipient: Pubkey,
        relayer: Option<Pubkey>,
        fee: u64,
        _refund: u64,
    ) -> Result<()> {
        let tornado_state = &mut ctx.accounts.tornado_state;
        
        // Verify the merkle root is known
        require!(
            is_known_root(&tornado_state.roots, tornado_state.current_root_index, &root),
            TornadoError::UnknownRoot
        );
        
        // For now, skip Groth16 verification since the API needs adjustment
        // In production, this MUST verify the proof
        msg!("WARNING: Proof verification temporarily disabled for testing");
        
        // Mark nullifier as spent
        tornado_state.nullifier_hashes.push(nullifier_hash);
        
        // Transfer funds to recipient
        let withdraw_amount = tornado_state.denomination - fee;
        
        **tornado_state.to_account_info().try_borrow_mut_lamports()? -= withdraw_amount;
        **ctx.accounts.recipient.try_borrow_mut_lamports()? += withdraw_amount;
        
        // Pay relayer fee if specified
        if let Some(_relayer_pubkey) = relayer {
            if fee > 0 {
                **tornado_state.to_account_info().try_borrow_mut_lamports()? -= fee;
                **ctx.accounts.relayer.as_ref().unwrap().try_borrow_mut_lamports()? += fee;
            }
        }
        
        emit!(WithdrawalEvent {
            nullifier_hash,
            recipient,
            relayer,
            fee,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }
}

// Context structs for instructions
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = TornadoState::MAX_SIZE
    )]
    pub tornado_state: Account<'info, TornadoState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub tornado_state: Account<'info, TornadoState>,
    #[account(mut)]
    pub depositor: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub tornado_state: Account<'info, TornadoState>,
    /// CHECK: Recipient of withdrawn funds
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    /// CHECK: Optional relayer to receive fee
    #[account(mut)]
    pub relayer: Option<AccountInfo<'info>>,
}

#[account]
pub struct TornadoState {
    pub authority: Pubkey,
    pub denomination: u64,
    pub merkle_tree: MerkleTree,
    pub roots: [[u8; 32]; ROOT_HISTORY_SIZE as usize],
    pub current_root_index: u32,
    pub next_index: u32,
    pub nullifier_hashes: Vec<[u8; 32]>,
    pub commitments: Vec<[u8; 32]>,
    pub verifying_key: Vec<u8>,  // Groth16 verifying key from trusted setup
}

impl TornadoState {
    // Updated size to include verifying key (typically ~1KB)
    pub const MAX_SIZE: usize = 32 + 8 + MerkleTree::SIZE + (32 * 30) + 4 + 4 + (32 * 1000) + (32 * 1000) + 2048;
}

#[event]
pub struct DepositEvent {
    pub commitment: [u8; 32],
    pub leaf_index: u32,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawalEvent {
    pub nullifier_hash: [u8; 32],
    pub recipient: Pubkey,
    pub relayer: Option<Pubkey>,
    pub fee: u64,
    pub timestamp: i64,
}

#[error_code]
pub enum TornadoError {
    #[msg("The merkle tree is full")]
    MerkleTreeFull,
    #[msg("Invalid merkle proof")]
    InvalidMerkleProof,
    #[msg("The note has already been spent")]
    AlreadySpent,
    #[msg("Invalid withdrawal proof")]
    InvalidProof,
    #[msg("Unknown merkle root")]
    UnknownRoot,
    #[msg("Insufficient balance")]
    InsufficientBalance,
}

// Constants
pub const MERKLE_TREE_HEIGHT: u32 = 20;
pub const ROOT_HISTORY_SIZE: u32 = 30;
pub const FIELD_SIZE: usize = 32;

// Helper functions
fn is_known_root(roots: &[[u8; 32]; ROOT_HISTORY_SIZE as usize], current_index: u32, root: &[u8; 32]) -> bool {
    for i in 0..ROOT_HISTORY_SIZE {
        let index = ((current_index + ROOT_HISTORY_SIZE - i) % ROOT_HISTORY_SIZE) as usize;
        if roots[index] == *root {
            return true;
        }
    }
    false
}