use tokio::task;
#[path = "auction_server/blockchain.rs"]
mod blockchain;
#[path = "auction_server/helper.rs"]
mod helper;
use helper::auctions_validator;
#[path = "auction_app/auction.rs"]
mod auction_app;
#[path = "auction_server/server.rs"]
mod server;
use crate::blockchain::{
    get_remote_blockchain, init_blockchain, save_blockchain_locally, Blockchain,
};
use server::{auction_server, retrieve_auction_house, retrieve_blockchain};
use std::env;

async fn blockchain_operator(blockchain: Blockchain) {
    let task1 = task::spawn(auction_server());
    let task2 = task::spawn(auctions_validator(blockchain));
    let task3 = task::spawn(retrieve_blockchain());
    let task4 = task::spawn(retrieve_auction_house());
    task1.await.unwrap();
    task2.await.unwrap();
    task3.await.unwrap();
    task4.await.unwrap();
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args[1] == "init_blockchain" {
        println!("init blockchain with genesis block");
        let bchain = init_blockchain().await;
        save_blockchain_locally(&bchain).await;
        blockchain_operator(bchain).await;
    } else {
        let bchain: Blockchain = get_remote_blockchain(&args[2].to_string()).await;
        println!("{:?}", bchain);
        save_blockchain_locally(&bchain).await;
        blockchain_operator(bchain).await;
    }
}
