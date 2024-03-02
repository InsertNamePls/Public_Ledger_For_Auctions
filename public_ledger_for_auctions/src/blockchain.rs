use std::process::exit;

use sha2::{Digest, Sha256};

use crate::DIFICULTY;

#[derive(Debug)]
pub struct Block {
    pub index: u32,
    pub prev_hash: String,
    pub nounce: u64,
    pub timestamp: i64,
    pub hash: String,
    pub tx: String
}

pub struct  Blockchain {
    pub blocks: Vec<Block>
}

impl Block{
    pub fn new(&self) {}
    pub fn gen_hash(&self) -> String{

        let blk_data = serde_json::json!({
            "index": self.index,
            "prev_hash": self.prev_hash,
            "hash": self.hash,
            "nounce": self.nounce,
            "timestamp": self.timestamp,
        }
        );
        let mut hasher = Sha256::new();
        hasher.update(blk_data.to_string());
        let result = hasher.finalize();
        result.iter().map(|b| format!("{:02x}", b)).collect::<String>()
    }

    pub fn mine_block(&mut self, dificulty: usize){
        let target = "0".repeat(dificulty);
        loop {
            if !self.hash.starts_with(&target) {
                self.nounce += 1;
                self.hash = self.gen_hash();
            }else {
                break;
            }

        }
    }
}

impl Blockchain {
     pub fn new(&self) {}
     pub fn add_block(&mut self, block: Block){
         self.blocks.push(block);
     }
}

pub fn validate_block(new_block: &Block, previous_block: &Block, dificulty: usize) {
    if previous_block.hash != new_block.prev_hash {
        println!("previous block hash does not match with new block previous hash.\n expected previous_hash={} got {}",previous_block.hash, new_block.prev_hash);
        exit(0)
    }else if !new_block.hash.starts_with(&"0".repeat(dificulty)) {
        println!("hash generated does not contain {} bits of difficulty",dificulty);
        exit(0)
    }
    else {
        println!("Block {} valid ",new_block.index);

    }
}
