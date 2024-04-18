use super::auction::*;
use crate::blockchain::Blockchain;
use crate::blockchain_operator::block_generator;
use crate::blockchain_pow::{
    block_peer_validator_client, blockchain_handler, blockchain_operator_validator,
};
use async_std::task::sleep;
use chrono::Utc;
use std::fs;
use std::vec::Vec;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
// Function executed in loop to validate if auctions is closed (10 in 10 seconds)
// if aution is closed create transaction and prepare to generate block
pub async fn auctions_validator(
    dest_ip: String,
    share_blockchain_vector: Arc<Mutex<Vec<Blockchain>>>,
) {
    let mut byte_count = 0;
    let mut tx: Vec<String> = Vec::new();

    loop {
        let data = fs::read_to_string("auction_data.json").expect("Unable to read file");
        let mut auction_house: AuctionHouse =
            serde_json::from_str(&data).expect("Failed to deserialize JSON");

        let mut blockchain_vector = share_blockchain_vector.lock().await;
        for auction in auction_house.auctions.iter_mut() {
            if &auction.end_time < &Utc::now()
                && !tx.contains(&auction.signature)
                && !auction.bids.is_empty()
                && auction.active
            {
                // insert into transaction vector
                tx.push(auction.signature.to_string());

                auction.active = false;
                println!("auction expired -> {:?}", auction);
                byte_count += &auction.signature.as_bytes().len();
                println!("{}", byte_count);
                if byte_count >= 5 {
                    println!("active blockchains -> {:?}\n", blockchain_vector);

                    let new_block =
                        block_generator(blockchain_vector.clone().get(0).unwrap().clone(), tx)
                            .await;

                    let result_peer_validation =
                        block_peer_validator_client(new_block.clone(), dest_ip.clone()).await;

                    let (result_validation, blockchain_vector_update) =
                        blockchain_operator_validator(new_block.clone(), blockchain_vector.clone())
                            .await;

                    tx = Vec::new();
                    byte_count = 0;
                    if result_validation && result_peer_validation {
                        blockchain_vector.clear();
                        for bch in blockchain_vector_update {
                            blockchain_vector.push(bch);
                        }
                    }

                    blockchain_handler(blockchain_vector.clone()).await;
                }
            }
        }
        let serialized = serde_json::to_string_pretty(&auction_house).unwrap();
        fs::write("auction_data.json", serialized).expect("error writing acution data");
        sleep(Duration::from_secs(10)).await;
    }
}
