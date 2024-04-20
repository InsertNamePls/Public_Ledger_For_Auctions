use super::auction::*;
use crate::blockchain::Blockchain;
use crate::blockchain_operator::block_generator;
use crate::blockchain_pow::{block_handler, block_peer_validator_client, blockchain_handler};
use chrono::Utc;
use std::vec::Vec;

use std::sync::Arc;
use tokio::sync::Mutex;
// Function executed in loop to validate if auctions is closed (10 in 10 seconds)
// if aution is closed create transaction and prepare to generate block
pub async fn auctions_validator(
    dest_ip: String,
    shared_blockchain_vector: Arc<Mutex<Vec<Blockchain>>>,
    shared_auction_house: Arc<Mutex<AuctionHouse>>,
) {
    let mut byte_count = 0;
    let mut tx: Vec<String> = Vec::new();

    loop {
        let mut auction_house = shared_auction_house.lock().await;
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
                    let new_block = block_generator(shared_blockchain_vector.clone(), tx).await;
                    let result_peer_validation =
                        block_peer_validator_client(new_block.clone(), dest_ip.clone())
                            .await
                            .expect("error getting validation from peer");

                    let result_validation =
                        block_handler(&mut shared_blockchain_vector.clone(), new_block.clone())
                            .await;
                    tx = Vec::new();

                    if result_validation && result_peer_validation {
                        blockchain_handler(&mut shared_blockchain_vector.clone()).await;
                    }

                    byte_count = 0;
                }
            }
        }
    }
}
