use crate::merkle::Hash;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u128,
    pub nonce: u32,
    pub timestamp: DateTime<Utc>,
    pub hash: Hash,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: u128, nonce: u32) -> Self {
        let timestamp = Utc::now();
        let mut tx = Self {
            from,
            to,
            amount,
            nonce,
            timestamp,
            hash: Hash::from_string(""), // Temporary
        };
        
        // Calculate the actual hash
        tx.hash = tx.calculate_hash();
        tx
    }

    pub fn calculate_hash(&self) -> Hash {
        let data = format!(
            "{}{}{}{}{}",
            self.from, self.to, self.amount, self.nonce, self.timestamp.timestamp()
        );
        Hash::from_string(&data)
    }

    pub fn is_valid(&self) -> bool {
        self.hash == self.calculate_hash() && 
        !self.from.is_empty() && 
        !self.to.is_empty() &&
        self.from != self.to
    }

    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u32,
    pub timestamp: DateTime<Utc>,
    pub transactions: Vec<Transaction>,
    pub previous_hash: Hash,
    pub merkle_root: Hash,
    pub hash: Hash,
    pub nonce: u64,
}

impl Block {
    pub fn new(index: u32, transactions: Vec<Transaction>, previous_hash: Hash) -> Self {
        let timestamp = Utc::now();
        let merkle_root = Self::calculate_merkle_root(&transactions);
        
        let mut block = Self {
            index,
            timestamp,
            transactions,
            previous_hash,
            merkle_root,
            hash: Hash::from_string(""), // Temporary
            nonce: 0,
        };
        
        block.hash = block.calculate_hash();
        block
    }

    pub fn genesis() -> Self {
        let genesis_hash = Hash::from_string("genesis");
        Self::new(0, Vec::new(), genesis_hash)
    }

    pub fn calculate_hash(&self) -> Hash {
        let data = format!(
            "{}{}{}{}{}",
            self.index,
            self.timestamp.timestamp(),
            self.previous_hash.to_hex(),
            self.merkle_root.to_hex(),
            self.nonce
        );
        Hash::from_string(&data)
    }

    fn calculate_merkle_root(transactions: &[Transaction]) -> Hash {
        if transactions.is_empty() {
            return Hash::from_string("empty");
        }

        let mut tree = crate::merkle::FastMerkleTree::new();
        for tx in transactions {
            tree.add_leaf(tx.hash.clone());
        }
        tree.build();
        
        tree.get_root().cloned().unwrap_or_else(|| Hash::from_string("empty"))
    }

    pub fn mine_block(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        
        loop {
            self.hash = self.calculate_hash();
            if self.hash.to_hex().starts_with(&target) {
                break;
            }
            self.nonce += 1;
        }
        
        println!("Block mined: {} with nonce: {}", self.hash, self.nonce);
    }

    pub fn is_valid(&self, previous_block: Option<&Block>) -> bool {
        // Check if hash is correctly calculated
        if self.hash != self.calculate_hash() {
            return false;
        }

        // Check if merkle root is correct
        if self.merkle_root != Self::calculate_merkle_root(&self.transactions) {
            return false;
        }

        // Check if previous hash matches
        if let Some(prev_block) = previous_block {
            if self.previous_hash != prev_block.hash {
                return false;
            }
            if self.index != prev_block.index + 1 {
                return false;
            }
        } else if self.index != 0 {
            return false; // Genesis block should have index 0
        }

        // Check if all transactions are valid
        for tx in &self.transactions {
            if !tx.is_valid() {
                return false;
            }
        }

        true
    }

    pub fn get_transaction_proof(&self, tx_index: usize) -> Option<Vec<Hash>> {
        if tx_index >= self.transactions.len() {
            return None;
        }

        let mut tree = crate::merkle::FastMerkleTree::new();
        for tx in &self.transactions {
            tree.add_leaf(tx.hash.clone());
        }
        tree.build();
        
        tree.get_proof(tx_index)
    }

    pub fn verify_transaction_inclusion(&self, tx: &Transaction, proof: &[Hash], tx_index: usize) -> bool {
        let mut tree = crate::merkle::FastMerkleTree::new();
        for transaction in &self.transactions {
            tree.add_leaf(transaction.hash.clone());
        }
        tree.build();
        
        tree.verify_proof(&tx.hash, proof, tx_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            100,
            1
        );
        assert!(tx.is_valid());
    }

    #[test]
    fn test_block_creation() {
        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            100,
            1
        );
        let previous_hash = Hash::from_string("previous");
        let block = Block::new(1, vec![tx], previous_hash);
        assert_eq!(block.index, 1);
        assert_eq!(block.transactions.len(), 1);
    }

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis();
        assert_eq!(genesis.index, 0);
        assert!(genesis.transactions.is_empty());
    }

    #[test]
    fn test_block_validation() {
        let genesis = Block::genesis();
        assert!(genesis.is_valid(None));

        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            100,
            1
        );
        let block = Block::new(1, vec![tx], genesis.hash.clone());
        assert!(block.is_valid(Some(&genesis)));
    }
}
