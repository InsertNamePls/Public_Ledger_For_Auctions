use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha256::digest;
use std::fs;
use tokio::net::TcpStream;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
const DIFICULTY: usize = 4;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u32,
    pub prev_hash: String,
    pub nounce: u64,
    pub timestamp: i64,
    pub hash: String,
    pub tx: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

pub async fn block_generator(mut blockchain: Blockchain, tx: Vec<String>) -> Blockchain {
    let previous_block = blockchain.blocks.last().unwrap();
    println!("previous{:?}", previous_block);
    let mut block: Block = Block::new(
        previous_block.index + 1,
        previous_block.hash.clone(),
        0,
        Utc::now().timestamp_millis(),
        "".to_string(),
        tx,
    );

    block.mine_block(4);
    if validate_block(&block, previous_block, DIFICULTY) {
        blockchain.add_block(block);
        save_blockchain_locally(&blockchain.clone()).await;
    }
    blockchain
}

pub async fn save_blockchain_locally(blockchain: &Blockchain) {
    let chain_serialized = serde_json::to_string_pretty(&blockchain).unwrap();
    fs::write("blockchain.json", chain_serialized).expect("Unable to write file");
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

pub async fn get_remote_blockchain(dest_addr: &String) -> Blockchain {
    let blockchain = Blockchain { blocks: Vec::new() };
    if let Ok(mut stream) = TcpStream::connect(format!("{}:3002", &dest_addr)).await {
        let request = "get_blockchain";

        if let Err(e) = stream.write_all(request.as_bytes()).await {
            eprintln!("error requesting data: {}", e);
        }
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer).await {
            Ok(n) => {
                let result = String::from_utf8_lossy(&buffer[..n]);
                let blockchain: Blockchain =
                    serde_json::from_str(&result).expect("Failed to deserialize JSON");
                println!("got blockchain from network -> {:?}\n", blockchain);
                save_blockchain_locally(&blockchain).await;
                blockchain
            }
            Err(e) => {
                eprintln!("error reading from server {}", e);
                blockchain
            }
        }
    } else {
        eprintln!("error conneting to server");
        blockchain
    }
}
