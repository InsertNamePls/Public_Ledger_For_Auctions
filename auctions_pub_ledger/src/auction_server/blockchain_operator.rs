use crate::auction::get_files_in_directory;
use crate::blockchain::{self, validate_block, Block, Blockchain};
use crate::blockchain_pow::*;
use chrono::Utc;
use local_ip_address::local_ip;
use std::ops::Index;
use std::process::exit;
use std::{fs, result, usize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const DIFICULTY: usize = 4;

pub async fn block_generator(blockchain: &Blockchain, tx: Vec<String>) -> Block {
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

    println!("generated_block {:?}", block);
    block

    // if validate_block(&block, previous_block, DIFICULTY)
    //    && block_peer_validator_client(block.clone(), dest_addr).await
}

// based on local blockchains validate if block is valid
pub async fn validator(mut blockchain: Blockchain, block: Block) -> bool {
    if validate_block(&block, blockchain.blocks.last().unwrap(), DIFICULTY) {
        true
    } else {
        println!("validator function block {} is invalid ", block.index);
        false
    }
}

pub async fn retrieve_blockchain() {
    let listener = TcpListener::bind("0.0.0.0:3002").await.unwrap();

    while let Ok((mut socket, _)) = listener.accept().await {
        let blochchain_vector: Vec<Blockchain> = blockchain_vector_build().await;

        let mut buffer = [0; 2048];
        match socket.read(&mut buffer).await {
            Ok(0) => break,
            Ok(_n) => {
                for blockchain in blochchain_vector {
                    println!("Blochchain sent: {:?}", blockchain);
                    let blockchain_str: String =
                        serde_json::to_string(&blockchain).expect("Failed to deserialize JSON");

                    if let Err(e) = socket.write_all(blockchain_str.as_bytes()).await {
                        eprintln!("error sending data: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from buffer: {}", e);
            }
        }
    }
}

pub async fn get_remote_blockchain(dest_addr: &String) -> Vec<Blockchain> {
    let mut blockchain_vector: Vec<Blockchain> = Vec::new();
    let mut file_counter = 0;
    if let Ok(mut stream) = TcpStream::connect(format!("{}:3002", &dest_addr)).await {
        let request = "get_blockchain";
        if let Err(e) = stream.write_all(request.as_bytes()).await {
            eprintln!("error requesting data: {}", e);
        }
        let mut buffer = [0; 1024];
        loop {
            match stream.read(&mut buffer).await {
                Ok(n) if n == 0 => {
                    println!("end of stream");
                    break;
                }
                Ok(n) => {
                    let result = String::from_utf8_lossy(&buffer[..n]);
                    let blockchain: Blockchain =
                        serde_json::from_str(&result).expect("Failed to deserialize JSON");
                    println!("got blockchain from network -> {:?}\n", blockchain);
                    save_blockchain_locally(
                        &blockchain,
                        format!("blockchain_active/blockchain_{}.json", file_counter).as_str(),
                    )
                    .await;

                    file_counter += 1;
                    blockchain_vector.push(blockchain);
                }
                Err(e) => {
                    eprintln!("error reading from server {}", e);
                    break;
                }
            }
        }
    } else {
        eprintln!("error conneting to server");
    }
    blockchain_vector
}

pub async fn save_blockchain_locally(blockchain: &Blockchain, file_path: &str) {
    let chain_serialized = serde_json::to_string_pretty(&blockchain).unwrap();
    fs::write(file_path, chain_serialized).expect("Unable to write file");
}
