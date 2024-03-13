//use async_std::sync::Mutex;
use chrono::Utc;
use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
mod blockchain;
use blockchain::*;
use std::fs;
pub mod helper;
use crate::helper::{
    handle_connection, network_validate_block, save_blockchain_locally, server_block_validate,
};

const DIFICULTY: usize = 4;
async fn get_remote_blockchain(dest_addr: &String) -> Blockchain {
    let blockchain = Blockchain { blocks: Vec::new() };
    if let Ok(mut stream) = TcpStream::connect(format!("{}:3002", &dest_addr)).await {
        let request = "get_blockchain";

        if let Err(e) = stream.write_all(request.as_bytes()).await {
            //if let Err(e) = stream.write_all(request.as_bytes()) {
            eprintln!("error requesting data: {}", e);
        }
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer).await {
            Ok(n) => {
                let result = String::from_utf8_lossy(&buffer[..n]);
                println!("{}", result);
                let blockchain: Blockchain =
                    serde_json::from_str(&result).expect("Failed to deserialize JSON");
                println!("{:?}", blockchain);
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
pub async fn retrieve_blockchain() {
    let listener = TcpListener::bind("0.0.0.0:3002").await.unwrap();

    while let Ok((socket, _)) = listener.accept().await {
        let local_blockchain = fs::read_to_string("./blockchain.txt").expect("Unable to read file");
        handle_connection(socket, local_blockchain).await;
    }
}
async fn gen_block(
    index: u32,
    prev_hash: String,
    hash: String,
    nounce: u64,
    timestamp: i64,
    tx: Vec<String>,
) -> Block {
    let mut block = Block {
        index,
        prev_hash,
        hash,
        nounce,
        timestamp,
        tx,
    };

    block.new();
    block.mine_block(DIFICULTY);
    block
}

async fn block_operator(
    blockchain: &Blockchain,
    dest_addr: String,
    tx: &Vec<String>,
) -> (Block, bool) {
    println!("block operator!!!!!");

    let previous_block_str = blockchain.blocks.last().unwrap();
    let previous_block: Block =
        serde_json::from_str(&previous_block_str).expect("Failed to deserialize JSON");

    let new_block = gen_block(
        previous_block.index + 1,
        previous_block.hash.clone(),
        "".to_string(),
        0,
        Utc::now().timestamp_millis(),
        tx.to_owned(),
    )
    .await;
    println!("{:?}", new_block);

    if network_validate_block(&new_block, &dest_addr).await
        && validate_block(&new_block, &previous_block, DIFICULTY)
    {
        (new_block, true)
    } else {
        (new_block, false)
    }
}

pub async fn transaction_server(mut blockchain: Blockchain, dest_addr: String) {
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    let mut tx: Vec<String> = Vec::new();
    let mut byte_count = 0;

    while let Ok((mut socket, _)) = listener.accept().await {
        let mut buffer = [0; 1024];
        match socket.read(&mut buffer).await {
            Ok(n) => {
                if byte_count <= 100 {
                    let request = String::from_utf8_lossy(&buffer[..n]);
                    println!("{}", request);
                    tx.push(request.to_string());
                    println!("{}", byte_count);
                    println!(" still have room for new transactions {:?}", tx);
                    byte_count = byte_count + n;
                } else {
                    println!("create block {:?}", &tx);

                    let (new_block, vaidation_result) =
                        block_operator(&blockchain, dest_addr.clone(), &tx).await;
                    if vaidation_result {
                        let block_serialized = serde_json::to_string(&new_block).unwrap();
                        blockchain.add_block(&block_serialized);

                        save_blockchain_locally(&blockchain).await;
                    }
                    byte_count = 0;
                }
            }
            Err(e) => {
                eprintln!("Error reading from buffer: {}", e);
            }
        }
    }
}

pub async fn init_blockchain(mut blockchain: Blockchain) -> Blockchain {
    blockchain.new();

    let genesis_block = gen_block(
        0,
        "".to_string(),
        "".to_string(),
        0,
        Utc::now().timestamp_millis(),
        Vec::new(),
    );
    let block_serialized = serde_json::to_string(&genesis_block.await).unwrap();

    blockchain.add_block(&block_serialized);

    save_blockchain_locally(&blockchain).await;
    blockchain
}
pub async fn get_local_blockchain() -> Blockchain {
    let data = fs::read_to_string("./blockchain.txt").expect("Unable to read file");
    let local_blockchain: Blockchain =
        serde_json::from_str(&data).expect("Failed to deserialize JSON");
    local_blockchain
}

pub async fn blockchain_operator(blockchain: Blockchain, dest_addr: String) {
    // todo Create a function that get available nodes
    //
    let task_one = tokio::spawn(transaction_server(blockchain, dest_addr));
    let task_two = tokio::spawn(server_block_validate());
    let task_three = tokio::spawn(retrieve_blockchain());
    let _ = tokio::try_join!(task_one, task_two, task_three);
}
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let blockchain = Blockchain { blocks: Vec::new() };
    if args[1] == "init_blockchain" {
        println!("init blockchain with genesis block");
        let blockchain = init_blockchain(blockchain).await;
        blockchain_operator(blockchain, args[2].to_string()).await;
    } else if args[1] == "join_blockchain" {
        let blockchain: Blockchain = get_remote_blockchain(&args[2].to_string()).await;
        println!("{:?}", blockchain);
        save_blockchain_locally(&blockchain).await;
        blockchain_operator(blockchain, args[2].to_string()).await;
    }
}
