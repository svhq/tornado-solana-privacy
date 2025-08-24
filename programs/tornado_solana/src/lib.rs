use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;

pub mod merkle_tree;
pub mod poseidon_hash;
pub mod merkle_tree_poseidon;
pub mod tests;
use merkle_tree::*;
use merkle_tree_poseidon::*;

declare_id!("ToRNaDo1111111111111111111111111111111111111");

#[program]
pub mod tornado_solana {
    use super::*;

    /// Initialize a new Tornado pool with fixed denomination
    pub fn initialize(ctx: Context<Initialize>, denomination: u64) -> Result<()> {
        let tornado_state = &mut ctx.accounts.tornado_state;
        
        tornado_state.authority = ctx.accounts.authority.key();
        tornado_state.denomination = denomination;
        tornado_state.merkle_tree = MerkleTree::new();
        tornado_state.current_root_index = 0;
        tornado_state.next_index = 0;
        
        Ok(())
    }

    /// Deposit funds into the tornado pool
    /// @param commitment: Hash(nullifier + secret)
    pub fn deposit(ctx: Context<Deposit>, commitment: [u8; 32]) -> Result<()> {
        let tornado_state = &mut ctx.accounts.tornado_state;
        
        // Check commitment hasn't been submitted before
        require!(
            !tornado_state.commitments.contains(&commitment),
            TornadoError::DuplicateCommitment
        );
        
        // Transfer SOL to the vault
        let transfer_ix = system_instruction::transfer(
            &ctx.accounts.depositor.key(),
            &ctx.accounts.tornado_state.key(),
            tornado_state.denomination,
        );
        
        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
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
        tornado_state.current_root_index = (tornado_state.current_root_index + 1) % ROOT_HISTORY_SIZE;
        tornado_state.roots[tornado_state.current_root_index as usize] = new_root;
        
        emit!(DepositEvent {
            commitment,
            leaf_index,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }

    /// Withdraw funds with a zero-knowledge proof
    pub fn withdraw(
        ctx: Context<Withdraw>,
        proof: Vec<u8>,
        root: [u8; 32],
        nullifier_hash: [u8; 32],
        recipient: Pubkey,
        relayer: Option<Pubkey>,
        fee: u64,
        refund: u64,
    ) -> Result<()> {
        let tornado_state = &mut ctx.accounts.tornado_state;
        
        // Verify fee doesn't exceed denomination
        require!(fee <= tornado_state.denomination, TornadoError::FeeExceedsDenomination);
        
        // Check nullifier hasn't been spent
        require!(
            !tornado_state.nullifier_hashes.contains(&nullifier_hash),
            TornadoError::NoteAlreadySpent
        );
        
        // Verify root is in history
        require!(
            is_known_root(&tornado_state.roots, tornado_state.current_root_index, &root),
            TornadoError::UnknownRoot
        );
        
        // TODO: Verify the zero-knowledge proof
        // For now, we'll mock this - in production, use groth16-solana
        require!(verify_proof(&proof, &root, &nullifier_hash, &recipient, fee), TornadoError::InvalidProof);
        
        // Mark nullifier as spent
        tornado_state.nullifier_hashes.push(nullifier_hash);
        
        // Calculate withdrawal amount
        let amount = tornado_state.denomination - fee;
        
        // Transfer to recipient
        **tornado_state.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.recipient.try_borrow_mut_lamports()? += amount;
        
        // Pay relayer fee if present
        if let Some(relayer_pubkey) = relayer {
            if fee > 0 {
                **tornado_state.to_account_info().try_borrow_mut_lamports()? -= fee;
                **ctx.accounts.relayer.as_ref().unwrap().try_borrow_mut_lamports()? += fee;
            }
        }
        
        emit!(WithdrawalEvent {
            to: recipient,
            nullifier_hash,
            relayer,
            fee,
        });
        
        Ok(())
    }
}

// Constants matching original Tornado Cash
pub const ROOT_HISTORY_SIZE: u32 = 30;
pub const MERKLE_TREE_HEIGHT: u32 = 20;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + TornadoState::MAX_SIZE,
        seeds = [b"tornado"],
        bump
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
    
    /// CHECK: Optional relayer receiving fee
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
}

impl TornadoState {
    pub const MAX_SIZE: usize = 32 + 8 + MerkleTree::SIZE + (32 * 30) + 4 + 4 + (32 * 1000) + (32 * 1000);
}

#[event]
pub struct DepositEvent {
    pub commitment: [u8; 32],
    pub leaf_index: u32,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawalEvent {
    pub to: Pubkey,
    pub nullifier_hash: [u8; 32],
    pub relayer: Option<Pubkey>,
    pub fee: u64,
}

#[error_code]
pub enum TornadoError {
    #[msg("The commitment has been submitted")]
    DuplicateCommitment,
    #[msg("Fee exceeds transfer value")]
    FeeExceedsDenomination,
    #[msg("The note has been already spent")]
    NoteAlreadySpent,
    #[msg("Cannot find your merkle root")]
    UnknownRoot,
    #[msg("Invalid withdraw proof")]
    InvalidProof,
    #[msg("Merkle tree is full")]
    MerkleTreeFull,
}

// Helper functions
fn is_known_root(roots: &[[u8; 32]; ROOT_HISTORY_SIZE as usize], current_index: u32, root: &[u8; 32]) -> bool {
    if root == &[0u8; 32] {
        return false;
    }
    
    let mut i = current_index;
    loop {
        if &roots[i as usize] == root {
            return true;
        }
        
        if i == 0 {
            i = ROOT_HISTORY_SIZE - 1;
        } else {
            i -= 1;
        }
        
        if i == current_index {
            break;
        }
    }
    
    false
}

// Mock proof verification - replace with groth16-solana in production
fn verify_proof(
    _proof: &[u8],
    _root: &[u8; 32],
    _nullifier_hash: &[u8; 32],
    _recipient: &Pubkey,
    _fee: u64,
) -> bool {
    // TODO: Implement actual Groth16 verification
    // For now, return true for testing
    true
}