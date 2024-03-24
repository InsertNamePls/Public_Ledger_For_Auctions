use super::auction::*;
use crate::auction_client::send_transaction;
use crate::blockchain::Blockchain;
use crate::blockchain_operator::block_generator;
use crate::blockchain_pow::{
    block_peer_validator_client, blockchain_handler, blockchain_operator_validator,
    blockchain_vector_build,
};
use async_std::task::sleep;
use chrono::Utc;
use local_ip_address::local_ip;
use std::fs;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

// Function executed in loop to validate if auctions is closed (10 in 10 seconds)
// if aution is closed create transaction and prepare to generate block
pub async fn auctions_validator(dest_ip: String, mut blockchain_vector: Vec<Blockchain>) {
    let mut byte_count = 0;
    let mut tx: Vec<String> = Vec::new();

    loop {
        let data = fs::read_to_string("auction_data.json").expect("Unable to read file");
        let mut auction_house: AuctionHouse =
            serde_json::from_str(&data).expect("Failed to deserialize JSON");
        for auction in auction_house.auctions.iter_mut() {
            if &auction.end_time < &Utc::now()
                && !tx.contains(&auction.signature)
                && !auction.bids.is_empty()
                && auction.active
            {
                // insert into transaction vector
                tx.push(auction.signature.to_string());

                auction.active = false;

                byte_count += &auction.signature.as_bytes().len();
                println!("{}", byte_count);
                if byte_count >= 5 {
                    println!("active blockchains {:?}", blockchain_vector);

                    //blockchain_vector = blockchain_vector_build().await;

                    println!(
                        "longest blockchain available: {:?}",
                        blockchain_vector.get(0).unwrap()
                    );
                    let new_block = block_generator(blockchain_vector.get(0).unwrap(), tx).await;

                    block_peer_validator_client(new_block.clone(), dest_ip.clone()).await;
                    let (_, blockchain_vector_update) =
                        blockchain_operator_validator(new_block.clone(), blockchain_vector).await;
                    blockchain_vector = blockchain_vector_update;
                    tx = Vec::new();
                    byte_count = 0;

                    blockchain_vector = blockchain_handler(blockchain_vector).await;
                }
            }
        }
        let serialized = serde_json::to_string_pretty(&auction_house).unwrap();
        fs::write("auction_data.json", serialized);
        sleep(Duration::from_secs(10)).await;
    }
}
