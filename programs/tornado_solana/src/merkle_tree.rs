use anchor_lang::prelude::*;
use sha3::{Digest, Keccak256};

/// Direct translation of MerkleTreeWithHistory from Tornado Cash
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MerkleTree {
    pub levels: u32,
    pub filled_subtrees: [[u8; 32]; 20],
    pub zeros: [[u8; 32]; 20],
    pub current_root: [u8; 32],
    pub next_index: u32,
}

impl MerkleTree {
    pub const SIZE: usize = 4 + (32 * 20) + (32 * 20) + 32 + 4;
    
    pub fn new() -> Self {
        let zeros = Self::generate_zeros();
        let mut filled_subtrees = [[0u8; 32]; 20];
        
        // Initialize with zero values
        for i in 0..20 {
            filled_subtrees[i] = zeros[i];
        }
        
        Self {
            levels: 20,
            filled_subtrees,
            zeros,
            current_root: zeros[19],
            next_index: 0,
        }
    }
    
    /// Generate zero values for empty leaves (matching Tornado Cash)
    fn generate_zeros() -> [[u8; 32]; 20] {
        let mut zeros = [[0u8; 32]; 20];
        
        // Start with keccak256("tornado") % FIELD_SIZE
        // Simplified for Solana (using full hash)
        zeros[0] = Self::hash_leaf(&[0u8; 32]);
        
        for i in 1..20 {
            zeros[i] = Self::hash_left_right(&zeros[i - 1], &zeros[i - 1]);
        }
        
        zeros
    }
    
    /// Insert a leaf into the merkle tree
    pub fn insert(&mut self, leaf: [u8; 32]) -> Result<u32> {
        require!(
            self.next_index < 2_u32.pow(self.levels),
            crate::TornadoError::MerkleTreeFull
        );
        
        let mut current_index = self.next_index;
        let mut current_level_hash = leaf;
        let mut left;
        let mut right;
        
        for i in 0..self.levels as usize {
            if current_index % 2 == 0 {
                left = current_level_hash;
                right = self.zeros[i];
                self.filled_subtrees[i] = current_level_hash;
            } else {
                left = self.filled_subtrees[i];
                right = current_level_hash;
            }
            
            current_level_hash = Self::hash_left_right(&left, &right);
            current_index /= 2;
        }
        
        self.current_root = current_level_hash;
        let inserted_index = self.next_index;
        self.next_index += 1;
        
        Ok(inserted_index)
    }
    
    /// Get the current merkle root
    pub fn get_root(&self) -> [u8; 32] {
        self.current_root
    }
    
    /// Hash two nodes together (equivalent to MiMC in original)
    /// Using Keccak256 for simplicity - replace with Poseidon later
    pub fn hash_left_right(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Keccak256::new();
        hasher.update(left);
        hasher.update(right);
        let result = hasher.finalize();
        
        let mut output = [0u8; 32];
        output.copy_from_slice(&result);
        output
    }
    
    /// Hash a single leaf
    pub fn hash_leaf(data: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Keccak256::new();
        hasher.update(data);
        let result = hasher.finalize();
        
        let mut output = [0u8; 32];
        output.copy_from_slice(&result);
        output
    }
    
    /// Generate merkle proof for a given leaf
    pub fn get_proof(&self, leaf_index: u32) -> Vec<[u8; 32]> {
        let mut proof = Vec::new();
        let mut index = leaf_index;
        
        for i in 0..self.levels as usize {
            if index % 2 == 0 {
                // If even, sibling is on the right
                if index + 1 < 2_u32.pow(i as u32 + 1) {
                    proof.push(self.zeros[i]);
                } else {
                    proof.push(self.zeros[i]);
                }
            } else {
                // If odd, sibling is on the left
                proof.push(self.filled_subtrees[i]);
            }
            index /= 2;
        }
        
        proof
    }
    
    /// Verify a merkle proof
    pub fn verify_proof(
        root: &[u8; 32],
        leaf: &[u8; 32],
        proof: &[[u8; 32]],
        index: u32,
    ) -> bool {
        let mut computed_hash = *leaf;
        let mut current_index = index;
        
        for sibling in proof {
            if current_index % 2 == 0 {
                computed_hash = Self::hash_left_right(&computed_hash, sibling);
            } else {
                computed_hash = Self::hash_left_right(sibling, &computed_hash);
            }
            current_index /= 2;
        }
        
        &computed_hash == root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_merkle_tree_insertion() {
        let mut tree = MerkleTree::new();
        
        let leaf1 = [1u8; 32];
        let leaf2 = [2u8; 32];
        
        let index1 = tree.insert(leaf1).unwrap();
        assert_eq!(index1, 0);
        
        let root1 = tree.get_root();
        
        let index2 = tree.insert(leaf2).unwrap();
        assert_eq!(index2, 1);
        
        let root2 = tree.get_root();
        assert_ne!(root1, root2);
    }
    
    #[test]
    fn test_merkle_proof() {
        let mut tree = MerkleTree::new();
        
        let leaf = [42u8; 32];
        let index = tree.insert(leaf).unwrap();
        let root = tree.get_root();
        
        let proof = tree.get_proof(index);
        assert!(MerkleTree::verify_proof(&root, &leaf, &proof, index));
    }
}