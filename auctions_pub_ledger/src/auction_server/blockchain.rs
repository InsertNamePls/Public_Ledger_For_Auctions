use chrono::Utc;
use colored::*;
use serde::{Deserialize, Serialize};
use sha256::digest;
use std::sync::Arc;
use std::usize;
use std::vec::Vec;
use tokio::sync::Mutex;
const DIFICULTY: usize = 4;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Block {
    pub index: u32,
    pub prev_hash: String,
    pub nounce: u64,
    pub timestamp: i64,
    pub hash: String,
    pub tx: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
}

impl Block {
    pub fn new(
        index: u32,
        prev_hash: String,
        nounce: u64,
        timestamp: i64,
        hash: String,
        tx: Vec<String>,
    ) -> Self {
        Block {
            index,
            prev_hash,
            nounce,
            timestamp,
            hash,
            tx,
        }
    }
    pub fn mine_block(&mut self, dificulty: usize) {
        let target = "0".repeat(dificulty);
        loop {
            if !self.hash.starts_with(&target) {
                self.nounce += 1;
                self.hash = gen_hash(
                    self.index,
                    self.prev_hash.clone(),
                    self.nounce,
                    self.timestamp,
                    self.tx.clone(),
                );
            } else {
                break;
            }
        }
    }
}
pub fn gen_hash(
    index: u32,
    prev_hash: String,
    nounce: u64,
    timestamp: i64,
    tx: Vec<String>,
) -> String {
    let blk_data = serde_json::json!({
        "index": index,
        "prev_hash": prev_hash,
        "nounce": nounce,
        "timestamp": timestamp,
        "tx": tx}
    );
    digest(blk_data.to_string())
}
impl Blockchain {
    pub fn new() -> Self {
        Blockchain { blocks: Vec::new() }
    }
    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
}

pub fn validate_block(new_block: &Block, previous_block: &Block, dificulty: usize) -> bool {
    if previous_block.hash != new_block.prev_hash {
        println!("previous block hash does not match with new block previous hash.\n expected previous_hash={} got {}",previous_block.hash, new_block.prev_hash);
        false
    } else if !new_block.hash.starts_with(&"0".repeat(dificulty)) {
        println!(
            "hash generated does not contain {} bits of difficulty",
            dificulty
        );
        false
    } else if previous_block.index + 1 != new_block.index {
        println!("Index of new block does not follow the previous_block");
        false
    } else if gen_hash(
        new_block.index,
        new_block.prev_hash.clone(),
        new_block.nounce,
        new_block.timestamp,
        new_block.tx.clone(),
    ) != new_block.hash
    {
        println!(
            "hash generated does not match with the one in the block {} ",
            new_block.hash
        );

        false
    } else {
        println!("{}", format!("Block {} valid\n", new_block.index).green());
        true
    }
}

pub async fn init_blockchain() -> Blockchain {
    let mut genesis_blk: Block = Block::new(
        0,
        "".to_string(),
        0,
        Utc::now().timestamp_millis(),
        "".to_string(),
        Vec::new(),
    );

    genesis_blk.mine_block(DIFICULTY);
    let mut blockchain: Blockchain = Blockchain::new();
    blockchain.add_block(genesis_blk);
    blockchain
}

pub async fn block_generator(
    shared_blockchain_vector: Arc<Mutex<Vec<Blockchain>>>,
    tx: Vec<String>,
) -> Block {
    let blockchain_vector = shared_blockchain_vector.lock().await;

    let main_blockchain = blockchain_vector.clone().get(0).unwrap().clone();

    let previous_block = main_blockchain.clone().blocks.last().unwrap().clone();
    println!("---------------------- Block Generator ----------------------\n");
    println!(
        "{}",
        format!(
            "Prevous Block\n index:{:?}\n hash:{:?} prev_hash:{:?}\n nounce:{:?}\n transactions{:?}\n",
            previous_block.index,
            previous_block.hash,
            previous_block.prev_hash,
            previous_block.nounce,
            previous_block.tx
        )
        .blue()
    );
    println!("-------------------------------------------------------------\n");

    let mut block: Block = Block::new(
        previous_block.index + 1,
        previous_block.hash.clone(),
        0,
        Utc::now().timestamp_millis(),
        "".to_string(),
        tx,
    );

    block.mine_block(4);
    println!(
        "{}",
        format!(
            "Generated Block\n index:{:?}\n hash:{:?} prev_hash:{:?}\n nounce:{:?}\n transactions{:?}\n",
            block.index, block.hash, block.prev_hash, block.nounce, block.tx
        )
        .green()
    );
    println!("-------------------------------------------------------------\n");

    block
}

// based on local blockchains validate if block is valid
pub async fn validator(blockchain: Blockchain, block: Block) -> bool {
    if validate_block(&block, blockchain.blocks.last().unwrap(), DIFICULTY) {
        true
    } else {
        println!("{}", format!("block {} is invalid ", block.index).red());
        false
    }
}
