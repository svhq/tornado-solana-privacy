use anchor_lang::prelude::*;
use light_poseidon::{Poseidon, PoseidonBytesHasher};
use ark_bn254::Fr;

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
    
    /// Generate zero values for empty leaves (Poseidon-based for circuit compatibility)
    fn generate_zeros() -> [[u8; 32]; 20] {
        let mut zeros = [[0u8; 32]; 20];
        
        // For Poseidon-based circuits, we use the hash of zero
        // This matches what circomlib's MerkleTree expects
        // The first zero is Poseidon(0)
        zeros[0] = Self::hash_leaf(&[0u8; 32]);
        
        // Each subsequent zero is the hash of two previous zeros
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
    
    /// Hash two nodes together using Poseidon (ZK-friendly)
    /// This is now using Light Protocol's Poseidon implementation
    pub fn hash_left_right(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        // Create a Poseidon hasher for 2 inputs
        let mut hasher = match Poseidon::<Fr>::new_circom(2) {
            Ok(h) => h,
            Err(_) => {
                // Fallback in case of initialization error
                // This should never happen with valid parameters
                return [0u8; 32];
            }
        };
        
        // Hash the two 32-byte inputs
        match hasher.hash_bytes_be(&[left, right]) {
            Ok(hash) => hash,
            Err(_) => {
                // Fallback in case of hashing error
                // This should only happen if inputs are invalid
                [0u8; 32]
            }
        }
    }
    
    /// Hash a single leaf using Poseidon
    pub fn hash_leaf(data: &[u8; 32]) -> [u8; 32] {
        // For a single input, we use Poseidon with 1 input
        let mut hasher = match Poseidon::<Fr>::new_circom(1) {
            Ok(h) => h,
            Err(_) => return [0u8; 32],
        };
        
        match hasher.hash_bytes_be(&[data]) {
            Ok(hash) => hash,
            Err(_) => [0u8; 32]
        }
    }
    
    /// Generate merkle proof for a given leaf (siblings only)
    pub fn get_proof(&self, leaf_index: u32) -> Vec<[u8; 32]> {
        let mut proof = Vec::new();
        let mut index = leaf_index;
        
        for i in 0..self.levels as usize {
            if index % 2 == 0 {
                // If even, sibling is on the right
                // For even nodes, we need to check if a right sibling exists
                // This happens when there's another leaf at index + 1
                proof.push(self.zeros[i]);
            } else {
                // If odd, sibling is on the left (always exists and is filled)
                proof.push(self.filled_subtrees[i]);
            }
            index /= 2;
        }
        
        proof
    }
    
    /// Generate merkle path with siblings and direction bits
    /// Returns (siblings, path_bits) where path_bits[i] = true if going right at level i
    pub fn get_path(&self, leaf_index: u32) -> (Vec<[u8; 32]>, Vec<bool>) {
        let mut siblings = Vec::new();
        let mut path_bits = Vec::new();
        let mut index = leaf_index;
        
        for i in 0..self.levels as usize {
            // Path bit indicates if we're the right child (1) or left child (0)
            let is_right_child = index % 2 == 1;
            path_bits.push(is_right_child);
            
            if is_right_child {
                // We're the right child, sibling is on the left
                siblings.push(self.filled_subtrees[i]);
            } else {
                // We're the left child, sibling is on the right
                siblings.push(self.zeros[i]);
            }
            index /= 2;
        }
        
        (siblings, path_bits)
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
    
    #[test]
    fn test_get_path_with_bits() {
        let mut tree = MerkleTree::new();
        
        // Insert multiple leaves to test different paths
        let leaf1 = [1u8; 32];
        let leaf2 = [2u8; 32];
        let leaf3 = [3u8; 32];
        
        tree.insert(leaf1).unwrap();
        tree.insert(leaf2).unwrap();
        let index3 = tree.insert(leaf3).unwrap();
        
        // Get path for third leaf (index 2)
        let (siblings, path_bits) = tree.get_path(index3);
        
        // Verify we have the correct number of elements
        assert_eq!(siblings.len(), 20);
        assert_eq!(path_bits.len(), 20);
        
        // Index 2 in binary is 0010, so:
        // Level 0: index 2 % 2 = 0 (left child)
        // Level 1: index 1 % 2 = 1 (right child)
        // Level 2: index 0 % 2 = 0 (left child)
        assert_eq!(path_bits[0], false); // left at level 0
        assert_eq!(path_bits[1], true);  // right at level 1
        assert_eq!(path_bits[2], false); // left at level 2
    }
    
    #[test]
    fn test_zero_values_poseidon() {
        let tree = MerkleTree::new();
        
        // Verify first zero is Poseidon(0)
        let expected_first_zero = MerkleTree::hash_leaf(&[0u8; 32]);
        assert_eq!(tree.zeros[0], expected_first_zero);
        
        // Verify each subsequent zero is hash of previous two
        for i in 1..20 {
            let expected = MerkleTree::hash_left_right(&tree.zeros[i-1], &tree.zeros[i-1]);
            assert_eq!(tree.zeros[i], expected);
        }
    }
    
    #[test]
    fn test_proof_verification_multiple_leaves() {
        let mut tree = MerkleTree::new();
        
        // Insert 4 leaves
        let leaves = [[1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32]];
        let mut indices = Vec::new();
        
        for leaf in &leaves {
            indices.push(tree.insert(*leaf).unwrap());
        }
        
        let root = tree.get_root();
        
        // Verify proof for each leaf
        for (i, leaf) in leaves.iter().enumerate() {
            let proof = tree.get_proof(indices[i]);
            assert!(
                MerkleTree::verify_proof(&root, leaf, &proof, indices[i]),
                "Proof verification failed for leaf at index {}", indices[i]
            );
        }
    }
}