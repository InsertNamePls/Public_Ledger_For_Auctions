use super::auction_app::*;
use crate::blockchain::{block_generator, Blockchain};
use async_std::task::sleep;
use chrono::Utc;
use std::fs;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
pub async fn bid_handler(
    target_auction: &Auction,
    mut auction_house: AuctionHouse,
    bid: Bid,
    mut socket: TcpStream,
) -> AuctionHouse {
    let mut last_highest_bid = 0.0;
    if !target_auction.bids.is_empty() {
        last_highest_bid = target_auction.bids[target_auction.bids.len() - 1].amount;
    }

    if &target_auction.end_time > &Utc::now() && last_highest_bid < bid.amount {
        auction_house
            .auctions
            .entry(bid.auction_id)
            .and_modify(|auction| auction.bids.push(bid.clone()));

        //save_auction_data(&auction_house).expect("erro saving auction file");
        let serialized = serde_json::to_string_pretty(&auction_house).unwrap();
        fs::write("auction_data.json", serialized);
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

pub async fn auctions_validator(mut blockchain: Blockchain) {
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
                && auction.active
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
        let serialized = serde_json::to_string_pretty(&auction_house).unwrap();
        fs::write("auction_data.json", serialized);
        //save_auction_data(&auction_house).expect("erro saving auction file");
        sleep(Duration::from_secs(10)).await;
    }
}
