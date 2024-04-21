use auction::AuctionHouse;
use tokio::task;
#[path = "auction_app/auction.rs"]
mod auction;
#[path = "auction_server/blockchain.rs"]
mod blockchain;
use crate::blockchain::{init_blockchain, Blockchain};
#[path = "auction_app/auction_client.rs"]
mod auction_client;
#[path = "auction_server/blockchain_operator.rs"]
mod blockchain_operator;
use crate::blockchain_operator::{
    blockchain_server, get_remote_blockchain, save_blockchain_locally,
};
#[path = "auction_server/auction_server.rs"]
mod auction_server;
#[path = "auction_server/auction_validator.rs"]
mod auction_validator;
use crate::auction_validator::auctions_validator;
#[path = "auction_server/blockchain_pow.rs"]
mod blockchain_pow;
use crate::auction_server::auction_server;
use std::env;
use std::fs;
use std::sync::Arc;
use std::vec::Vec;
use tokio::sync::Mutex;
#[path = "cryptography/ecdsa_keys.rs"]
mod keys;
async fn destributed_auction_operator(blockchain_vector: Vec<Blockchain>, dest_ip: String) {
    let shared_blockchain_vector = Arc::new(Mutex::new(blockchain_vector));

    // initialize auction house by importing from file
    let data = fs::read_to_string("auction_data.json").expect("Unable to read file");
    let auction_house: AuctionHouse =
        serde_json::from_str(&data).expect("Failed to deserialize JSON");

    let share_auction_house = Arc::new(Mutex::new(auction_house));

    let task1 = task::spawn(auction_server(share_auction_house.clone()));
    let task2 = task::spawn(auctions_validator(
        dest_ip.clone(),
        shared_blockchain_vector.clone(),
        share_auction_house.clone(),
    ));
    let task3 = task::spawn(blockchain_server(shared_blockchain_vector.clone()));

    task1.await.unwrap();
    task2.await.unwrap();
    task3.await.unwrap();
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let dest_ip = args[2].to_string();
    if args[1] == "init_blockchain" {
        let mut blockchain_vector: Vec<Blockchain> = Vec::new();
        println!("init blockchain with genesis block");

        let bchain = init_blockchain().await;

        blockchain_vector.push(bchain.clone());
        save_blockchain_locally(&bchain, "blockchain_active/blockchain_0.json").await;
        destributed_auction_operator(blockchain_vector, dest_ip.clone()).await;
    } else {
        match get_remote_blockchain(args[2].to_string().clone()).await {
            Ok(result) => {
                println!("{:?}", result);
                destributed_auction_operator(result, dest_ip.clone()).await;
            }
            Err(e) => {
                println!("error {}", e);
            }
        }
    }
}
