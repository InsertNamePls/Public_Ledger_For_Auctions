use crate::blockchain::{Block, Blockchain};
use crate::blockchain_operator::{save_blockchain_locally, validator};
use chrono::Utc;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use tokio::sync::Mutex;

pub async fn block_peer_validator_client(block: Block, dest_addr: String) -> bool {
    let block_str = serde_json::to_string(&block).unwrap();
    if let Ok(mut stream) = TcpStream::connect(format!("{}:3001", dest_addr)).await {
        if let Err(e) = stream.write_all(block_str.as_bytes()).await {
            eprintln!("error requesting data: {}", e);
        }
        let mut buffer = [0; 2048];
        match stream.read(&mut buffer).await {
            Ok(n) => {
                let result = String::from_utf8_lossy(&buffer[..n]);
                println!("\nresult validation from peed ->{}", result);
                if result == "true" {
                    true
                } else {
                    false
                }
            }
            Err(e) => {
                eprintln!("error reading from server {}", e);
                false
            }
        }
    } else {
        eprintln!("error conneting to server");
        false
    }
}

pub async fn block_peer_validator_server(shared_blockchain_vector: Arc<Mutex<Vec<Blockchain>>>) {
    let listener = TcpListener::bind("0.0.0.0:3001").await.unwrap();
    while let Ok((mut socket, _)) = listener.accept().await {
        let mut buffer = [0; 2048];

        match socket.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                let mut blockchain_vector = shared_blockchain_vector.lock().await;
                let result = String::from_utf8_lossy(&buffer[..n]);
                let incoming_block: Block =
                    serde_json::from_str(&result).expect("Failed to deserialize JSON");
                println!(
                    "Incoming block from peer-> {:?} {:?}\n",
                    socket.peer_addr().unwrap(),
                    incoming_block
                );

                let (result, blockchain_vector_update) =
                    blockchain_operator_validator(incoming_block, blockchain_vector.clone()).await;
                blockchain_vector.clear();
                if result {
                    for bch in blockchain_vector_update {
                        blockchain_vector.push(bch);
                    }
                }

                if let Err(e) = socket.write_all(result.to_string().as_bytes()).await {
                    eprintln!("error sending data: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Error reading from buffer: {}", e);
            }
        }
    }
}

pub async fn blockchain_operator_validator(
    incoming_block: Block,
    blockchain_vector: Vec<Blockchain>,
) -> (bool, Vec<Blockchain>) {
    let (result, update_active_blockchains) =
        block_handler(blockchain_vector, incoming_block.clone()).await;
    blockchain_store(
        update_active_blockchains.clone(),
        "blockchain_active/blockchain",
    )
    .await;

    (result, update_active_blockchains)
}
pub async fn block_handler(
    mut active_blockchains: Vec<Blockchain>,
    block: Block,
) -> (bool, Vec<Blockchain>) {
    let mut validator_result = false;
    for (i, blockchain) in active_blockchains.clone().iter().enumerate() {
        if blockchain.blocks.last().unwrap().index == block.clone().index - 1 {
            println!("\nUsing main branch");
            validator_result = validator(blockchain.clone(), block.clone()).await;
            if validator_result {
                if let Some(target_blockchain) = active_blockchains.get_mut(i) {
                    target_blockchain.blocks.push(block.clone());
                }
                println!("updated blockchain {:?}", blockchain);
                println!("active blockchains {:?}", active_blockchains);
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
    (validator_result, active_blockchains)
}

pub async fn blockchain_handler(blockchain_vector: Vec<Blockchain>) -> Vec<Blockchain> {
    let mut biggest_blockchain_len = 0;
    for blockchain in blockchain_vector.clone() {
        if blockchain.blocks.len() > biggest_blockchain_len {
            biggest_blockchain_len = blockchain.blocks.len()
        }
    }
    println!(
        "Biggest blockchain in the system has {:?} blocks",
        biggest_blockchain_len
    );

    let (mut active_blockchains, archive_blockchains): (Vec<Blockchain>, Vec<Blockchain>) =
        blockchain_vector.into_iter().partition(|blockchain| {
            blockchain.blocks.len()
                >= i32::abs(biggest_blockchain_len as i32 - 2)
                    .abs()
                    .try_into()
                    .unwrap()
        });
    println!("\nActive blockchains: {:?} \n", active_blockchains);

    let timestamp = Utc::now().timestamp_millis();
    blockchain_store(
        archive_blockchains,
        format!("blockchain_archive/blockchain_{}", timestamp).as_str(),
    )
    .await;
    active_blockchains.sort_by_key(|bch| bch.blocks.len());

    active_blockchains
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
