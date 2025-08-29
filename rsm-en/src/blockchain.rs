use crate::merkle::{Hash, FastMerkleTree};
use crate::transaction::{Transaction, Block};
use crate::system::Pallet as SystemPallet;
use crate::balances::Pallet as BalancesPallet;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: usize,
    pub pending_transactions: VecDeque<Transaction>,
    pub mining_reward: u128,
    pub system: SystemPallet,
    pub balances: BalancesPallet,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Self {
            chain: Vec::new(),
            difficulty: 2,
            pending_transactions: VecDeque::new(),
            mining_reward: 100,
            system: SystemPallet::new(),
            balances: BalancesPallet::new(),
        };
        
        // Create genesis block
        blockchain.create_genesis_block();
        blockchain
    }

    fn create_genesis_block(&mut self) {
        let mut genesis = Block::genesis();
        genesis.mine_block(self.difficulty);
        self.chain.push(genesis);
        
        // Initialize some accounts with genesis balances
        self.balances.set_balance(&"genesis".to_string(), 1_000_000);
        self.balances.set_balance(&"alice".to_string(), 1000);
        self.balances.set_balance(&"bob".to_string(), 500);
    }

    pub fn get_latest_block(&self) -> &Block {
        self.chain.last().expect("Chain should have at least genesis block")
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
        if !transaction.is_valid() {
            return Err("Invalid transaction".to_string());
        }

        // Check if sender has sufficient balance
        let sender_balance = self.balances.get_balance(&transaction.from);
        if sender_balance < transaction.amount {
            return Err("Insufficient balance".to_string());
        }

        // Check nonce
        let expected_nonce = self.system.get_nonce(&transaction.from);
        if transaction.nonce != expected_nonce + 1 {
            return Err("Invalid nonce".to_string());
        }

        self.pending_transactions.push_back(transaction);
        Ok(())
    }

    pub fn mine_pending_transactions(&mut self, mining_reward_address: String) -> Result<Block, String> {
        if self.pending_transactions.is_empty() {
            return Err("No pending transactions to mine".to_string());
        }

        // Add mining reward transaction
        let reward_tx = Transaction::new(
            "network".to_string(),
            mining_reward_address.clone(),
            self.mining_reward,
            0
        );

        let mut transactions = Vec::new();
        transactions.push(reward_tx);

        // Process pending transactions
        while let Some(tx) = self.pending_transactions.pop_front() {
            // Execute the transaction
            match self.balances.transfer(
                tx.from.clone(),
                tx.to.clone(),
                tx.amount
            ) {
                Ok(_) => {
                    self.system.inc_nonce(&tx.from);
                    transactions.push(tx);
                }
                Err(e) => {
                    println!("Transaction failed: {}", e);
                    // Skip invalid transaction
                }
            }

            // Limit transactions per block
            if transactions.len() >= 100 {
                break;
            }
        }

        let previous_hash = self.get_latest_block().hash.clone();
        let mut new_block = Block::new(
            self.chain.len() as u32,
            transactions,
            previous_hash
        );

        new_block.mine_block(self.difficulty);

        // Add mining reward to the miner's balance
        let current_balance = self.balances.get_balance(&mining_reward_address);
        self.balances.set_balance(
            &mining_reward_address,
            current_balance + self.mining_reward
        );

        // Increment block number
        self.system.inc_block_number(&mining_reward_address);

        self.chain.push(new_block.clone());
        Ok(new_block)
    }

    pub fn get_balance(&mut self, address: &String) -> u128 {
        self.balances.get_balance(address)
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if !current_block.is_valid(Some(previous_block)) {
                return false;
            }

            // Check proof of work
            let target = "0".repeat(self.difficulty);
            if !current_block.hash.to_hex().starts_with(&target) {
                return false;
            }
        }
        true
    }

    pub fn get_transaction_history(&self, address: &String) -> Vec<&Transaction> {
        let mut history = Vec::new();
        
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.from == *address || tx.to == *address {
                    history.push(tx);
                }
            }
        }
        
        history
    }

    pub fn find_transaction(&self, tx_hash: &Hash) -> Option<(&Block, &Transaction, usize)> {
        for block in &self.chain {
            for (index, tx) in block.transactions.iter().enumerate() {
                if tx.hash == *tx_hash {
                    return Some((block, tx, index));
                }
            }
        }
        None
    }

    pub fn get_transaction_proof(&self, tx_hash: &Hash) -> Option<(Vec<Hash>, usize, u32)> {
        if let Some((block, _tx, tx_index)) = self.find_transaction(tx_hash) {
            if let Some(proof) = block.get_transaction_proof(tx_index) {
                return Some((proof, tx_index, block.index));
            }
        }
        None
    }

    pub fn verify_transaction_proof(&self, tx: &Transaction, proof: &[Hash], tx_index: usize, block_index: u32) -> bool {
        if let Some(block) = self.chain.get(block_index as usize) {
            return block.verify_transaction_inclusion(tx, proof, tx_index);
        }
        false
    }

    pub fn get_block_by_index(&self, index: u32) -> Option<&Block> {
        self.chain.get(index as usize)
    }

    pub fn get_block_by_hash(&self, hash: &Hash) -> Option<&Block> {
        self.chain.iter().find(|block| block.hash == *hash)
    }

    pub fn get_chain_length(&self) -> usize {
        self.chain.len()
    }

    pub fn get_pending_transaction_count(&self) -> usize {
        self.pending_transactions.len()
    }

    pub fn set_difficulty(&mut self, difficulty: usize) {
        self.difficulty = difficulty;
    }

    pub fn get_network_hash_rate(&self) -> f64 {
        if self.chain.len() < 2 {
            return 0.0;
        }

        let latest_block = self.get_latest_block();
        let prev_block = &self.chain[self.chain.len() - 2];
        
        let time_diff = (latest_block.timestamp.timestamp() - prev_block.timestamp.timestamp()) as f64;
        let target_value = 2_u64.pow(self.difficulty as u32) as f64;
        
        if time_diff > 0.0 {
            target_value / time_diff
        } else {
            0.0
        }
    }

    pub fn create_state_merkle_tree(&self) -> FastMerkleTree {
        let mut tree = FastMerkleTree::new();
        
        // Add all account balances to the tree
        for (account, balance) in &self.balances.balances {
            let state_data = format!("{}:{}", account, balance);
            tree.add_leaf(Hash::from_string(&state_data));
        }
        
        tree.build();
        tree
    }

    pub fn get_state_root(&self) -> Option<Hash> {
        let tree = self.create_state_merkle_tree();
        tree.get_root().cloned()
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.chain.len(), 1); // Genesis block
        assert!(blockchain.is_chain_valid());
    }

    #[test]
    fn test_add_transaction() {
        let mut blockchain = Blockchain::new();
        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            100,
            1
        );
        
        let result = blockchain.add_transaction(tx);
        assert!(result.is_ok());
        assert_eq!(blockchain.get_pending_transaction_count(), 1);
    }

    #[test]
    fn test_mine_block() {
        let mut blockchain = Blockchain::new();
        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            100,
            1
        );
        
        blockchain.add_transaction(tx).unwrap();
        let result = blockchain.mine_pending_transactions("miner".to_string());
        
        assert!(result.is_ok());
        assert_eq!(blockchain.chain.len(), 2);
        assert!(blockchain.is_chain_valid());
    }

    #[test]
    fn test_transaction_history() {
        let mut blockchain = Blockchain::new();
        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            100,
            1
        );
        
        blockchain.add_transaction(tx).unwrap();
        blockchain.mine_pending_transactions("miner".to_string()).unwrap();
        
        let alice_history = blockchain.get_transaction_history(&"alice".to_string());
        assert!(!alice_history.is_empty());
    }
}
