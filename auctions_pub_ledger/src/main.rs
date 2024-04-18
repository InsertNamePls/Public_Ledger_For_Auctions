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
    get_remote_blockchain, retrieve_blockchain, save_blockchain_locally,
};
#[path = "auction_server/auction_server.rs"]
mod auction_server;
#[path = "auction_server/auction_validator.rs"]
mod auction_validator;
use crate::auction_validator::auctions_validator;
#[path = "auction_server/blockchain_pow.rs"]
mod blockchain_pow;
use crate::auction_server::auction_server;
use crate::blockchain_pow::block_peer_validator_server;
use std::env;
use std::sync::Arc;
use std::vec::Vec;
use tokio::sync::Mutex;

#[path = "cryptography/ecdsa_keys.rs"]
mod keys;
//use crate::keys::ecdsa_keys;
async fn destributed_auction_operator(blockchain_vector: Vec<Blockchain>, dest_ip: String) {
    let shared_blockchain_vector = Arc::new(Mutex::new(blockchain_vector));

    let task1 = task::spawn(auction_server());
    let task2 = task::spawn(auctions_validator(
        dest_ip.clone(),
        shared_blockchain_vector.clone(),
    ));
    let task3 = task::spawn(retrieve_blockchain(shared_blockchain_vector.clone()));
    let task5 = task::spawn(block_peer_validator_server(
        shared_blockchain_vector.clone(),
    ));
    task1.await.unwrap();
    task2.await.unwrap();
    task3.await.unwrap();
    task5.await.unwrap();
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
        let blockchain_vector: Vec<Blockchain> =
            get_remote_blockchain(&args[2].to_string().clone()).await;
        println!("{:?}", blockchain_vector);
        //save_blockchain_locally(&bchain, "blockchain_active/blockchain0.json").await;
        destributed_auction_operator(blockchain_vector, dest_ip.clone()).await;
    }
}
