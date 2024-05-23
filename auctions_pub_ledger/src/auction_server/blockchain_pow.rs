use crate::auction_server::blockchain::validator;
use crate::auction_server::blockchain::{Block, Blockchain};
use crate::auction_server::blockchain_operator::save_blockchain_locally;
use chrono::Utc;
use std::sync::Arc;

use tokio::sync::Mutex;

pub async fn block_handler(
    shared_active_blockchains: &mut Arc<Mutex<Vec<Blockchain>>>,
    block: Block,
) -> bool {
    let mut validator_result = false;
    let mut active_blockchains = shared_active_blockchains.lock().await;
    for (i, blockchain) in active_blockchains.clone().iter().enumerate() {
        if blockchain.blocks.last().unwrap().index == block.clone().index - 1 {
            println!("\nUsing main branch");
            validator_result = validator(blockchain.clone(), block.clone()).await;
            if validator_result {
                if let Some(target_blockchain) = active_blockchains.get_mut(i) {
                    target_blockchain.blocks.push(block.clone());
                }
                println!("active blockchains {:?}\n", active_blockchains);
                break;
            }
        }
        if blockchain.blocks.last().unwrap().index - block.clone().index <= 2 {
            let mut forked_blockchain: Blockchain = Blockchain::new();
            let filtered_blocks: Vec<Block> = blockchain
                .blocks
                .clone()
                .into_iter()
                .enumerate()
                .filter(|(index, _)| *index < block.clone().index as usize)
                .map(|(_, item)| item)
                .collect();

            forked_blockchain.blocks = filtered_blocks;
            println!(
                "\nFork main blockchain! attempt to fork blockchain {:?}\n To add block {:?}\n",
                forked_blockchain,
                block.clone()
            );
            validator_result = validator(forked_blockchain.clone(), block.clone()).await;
            if validator_result {
                forked_blockchain.blocks.push(block.clone());
                active_blockchains.push(forked_blockchain);
                break;
            }
        }
    }
    validator_result
}

pub async fn blockchain_handler(shared_blockchain_vector: &mut Arc<Mutex<Vec<Blockchain>>>) {
    let mut biggest_blockchain_len = 0;

    let mut blockchain_vector = shared_blockchain_vector.lock().await;
    for blockchain in blockchain_vector.clone() {
        if blockchain.blocks.len() > biggest_blockchain_len {
            biggest_blockchain_len = blockchain.blocks.len()
        }
    }
    println!(
        "Biggest blockchain in the system has {:?} blocks\n",
        biggest_blockchain_len
    );
    // create 2 vectors that one contains the active blockchains and the other that has the
    // difference chain length > 2
    let (active_blockchains, archive_blockchains): (Vec<Blockchain>, Vec<Blockchain>) =
        blockchain_vector
            .clone()
            .into_iter()
            .partition(|blockchain| {
                blockchain.blocks.len()
                    >= i32::abs(biggest_blockchain_len as i32 - 2)
                        .abs()
                        .try_into()
                        .unwrap()
            });

    println!("\nActive blockchains: {:?} \n", active_blockchains);

    // if there are archivable blochainif archive_blockchains.clone().len()::MAX > 0 {
    if archive_blockchains.clone().len() > 0 {
        for y in archive_blockchains.clone() {
            blockchain_vector.retain(|blockchain| blockchain != &y);
        }
        blockchain_vector.sort_by_key(|bch| bch.blocks.len());
        //save archive blockchain in a log file
        let timestamp = Utc::now().timestamp_millis();
        blockchain_store(
            archive_blockchains.clone(),
            format!("blockchain_archive/blockchain_{}", timestamp).as_str(),
        )
        .await;
    }
    blockchain_store(blockchain_vector.clone(), "blockchain_active/blockchain").await;
}

pub async fn blockchain_store(blockchain_vector: Vec<Blockchain>, file_name_path: &str) {
    for (i, blockchain) in blockchain_vector.iter().enumerate() {
        save_blockchain_locally(
            &blockchain.clone(),
            format!("{}_{}.json", file_name_path, i).as_str(),
        )
        .await;
    }
}
