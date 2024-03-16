use std::time::Duration;

use async_std::task::sleep;
use auction_app::auction::Auction;
use chrono::Utc;
mod blockchain;
use crate::auction_app::auction::{save_auction_data, AuctionHouse, Bid, Transaction};
use crate::blockchain::{validate_block, Block, Blockchain};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
mod auction_app {
    pub mod auction;
}
use local_ip_address::local_ip;

use std::fs;
use tokio::task;

const DIFICULTY: usize = 4;

async fn auction_server() {
    let my_local_ip = local_ip().unwrap();
    let listener = TcpListener::bind(format!("{}:3000", my_local_ip))
        .await
        .unwrap();
    loop {
        while let Ok((mut socket, _)) = listener.accept().await {
            let mut buffer = [0; 2048];
            match socket.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => {
                    let request = String::from_utf8_lossy(&buffer[..n]);
                    let m: Transaction = serde_json::from_str(&request).unwrap();
                    let data =
                        fs::read_to_string("auction_data.json").expect("Unable to read file");
                    let mut auction_house: AuctionHouse =
                        serde_json::from_str(&data).expect("Failed to deserialize JSON");
                    match m {
                        Transaction::Bid(value) => {
                            println!("\n{:?}", value);

                            let target_auction = auction_house.auctions[&value.auction_id].clone();

                            auction_house = bid_handler(
                                &target_auction,
                                auction_house.clone(),
                                value.clone(),
                                socket,
                            )
                            .await;
                        }
                        Transaction::Auction(value) => {
                            println!("\n{:?}", auction_house);

                            auction_house
                                .add_auction(value, (auction_house.auctions.len() as u32) + 1);
                            save_auction_data(&auction_house).expect("erro saving auction file");
                        }
                    }
                    println!("{:?}", auction_house);
                }
                Err(e) => {
                    eprintln!("Error reading from buffer: {}", e);
                }
            }
        }
    }
}

async fn bid_handler(
    target_auction: &Auction,
    mut auction_house: AuctionHouse,
    bid: Bid,
    mut socket: TcpStream,
) -> AuctionHouse {
    let mut last_highest_bid = 0.0;
    match target_auction.bids.len() {
        0 => {
            last_highest_bid = 0.0;
        }
        _ => {
            last_highest_bid = target_auction.bids[target_auction.bids.len() - 1].amount;
        }
    }
    if &target_auction.end_time > &Utc::now() && last_highest_bid < bid.amount {
        auction_house
            .auctions
            .entry(bid.auction_id)
            .and_modify(|auction| auction.bids.push(bid.clone()));

        save_auction_data(&auction_house).expect("erro saving auction file");

        auction_house
    } else {
        socket
            .write_all(
                "action not available or the bid value is lower than the last bid".as_bytes(),
            )
            .await
            .expect("erro sending response");

        auction_house
    }
}
async fn auctions_validator(mut blockchain: Blockchain) {
    let mut byte_count = 0;
    let mut tx: Vec<String> = Vec::new();

    loop {
        let data = fs::read_to_string("auction_data.json").expect("Unable to read file");
        let mut auction_house: AuctionHouse =
            serde_json::from_str(&data).expect("Failed to deserialize JSON");
        for (auction_id, auction) in auction_house.clone().auctions.iter() {
            if &auction.end_time < &Utc::now()
                && !tx.contains(&auction.signature)
                && !auction.bids.is_empty()
                && auction.active == true
            {
                // insert into transaction vector
                tx.push(auction.signature.to_string());
                auction_house
                    .auctions
                    .entry(auction_id.to_owned())
                    .and_modify(|auction| auction.active = false);

                byte_count += &auction.signature.as_bytes().len();
                println!("{}", byte_count);
                if byte_count >= 5 {
                    blockchain = block_generator(blockchain, tx).await;

                    tx = Vec::new();
                    println!("{:?}", blockchain);
                    byte_count = 0;
                }
            }
        }

        save_auction_data(&auction_house).expect("erro saving auction file");
        sleep(Duration::from_secs(10)).await;
    }
}

async fn block_generator(mut blockchain: Blockchain, tx: Vec<String>) -> Blockchain {
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
#[tokio::main]
async fn main() {
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

    let handle2 = task::spawn(auction_server());

    let handle1 = task::spawn(auctions_validator(blockchain));
    handle1.await.unwrap();
    handle2.await.unwrap();
}
