use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, MutexGuard};

use crate::helper::handle_connection;
const DIFICULTY: usize = 4;

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub index: u32,
    pub prev_hash: String,
    pub nounce: u64,
    pub timestamp: i64,
    pub hash: String,
    pub tx: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
//pub struct Blockchain {
//    pub blocks: Vec<Block>,
//}
pub struct Blockchain {
    pub blocks: Vec<String>,
}
impl Block {
    pub fn new(&self) {}
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

impl Blockchain {
    pub fn new(&self) {}
    pub fn add_block(&mut self, block: &String) {
        self.blocks.push(block.to_string());
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
    let mut hasher = Sha256::new();
    hasher.update(blk_data.to_string());
    let result = hasher.finalize();
    result
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
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
        println!("Block {} valid ", new_block.index);
        true
    }
}
