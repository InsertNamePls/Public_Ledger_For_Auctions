use chrono::{Utc};

mod blockchain;
use blockchain::*;
const DIFICULTY: usize = 4;

fn main() {
    let mut genesis_blk=Block {
        index: 0,
        prev_hash: String::from(""),
        hash: String::from(""),
        nounce: 0,
        timestamp: Utc::now().timestamp_millis(),
        tx: format!("{} trasaction", 0)
    };
    let mut blockchain =Blockchain{
        blocks: Vec::new()
    };
    blockchain.new();
    genesis_blk.new();
    genesis_blk.mine_block(DIFICULTY);
    blockchain.add_block(genesis_blk);

    for i in 1..=5 {
        let previous_block = blockchain.blocks.last().unwrap();
        let mut new_block = Block{
            index:i,
            prev_hash: previous_block.prev_hash.clone(),
            nounce: 0,
            timestamp: Utc::now().timestamp_millis(),
            tx: format!("{} trasaction block ", i) as String,
            hash: String::from(""),

        };
        new_block.mine_block(DIFICULTY);
        blockchain.add_block(new_block);
    }
    println!("{:?}", blockchain.blocks)
}
