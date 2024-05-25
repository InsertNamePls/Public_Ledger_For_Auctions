use crate::auction_app::auction::{AuctionHouse, Bid, Transaction};
use crate::auction_app::auction_operation::client::send_transaction;
use crate::auction_app::notifications::notify_client::send_notification;
use chrono::Utc;
use elliptic_curve::generic_array::GenericArray;
use k256::ecdsa::Signature;
use k256::ecdsa::{signature::Verifier, VerifyingKey};
use sha256::digest;
use std::sync::Arc;
use tokio::sync::Mutex;
pub async fn transaction_handler(
    transaction: Transaction,
    shared_auction_house: &mut Arc<Mutex<AuctionHouse>>,
    requester_addr: String,
    routing_table: Vec<String>,
) {
    match transaction {
        Transaction::Bid(ref value) => {
            println!("\n{:?}", value);
            find_auction_to_bid(
                &mut shared_auction_house.clone(),
                value,
                transaction.clone(),
                requester_addr,
                routing_table.clone(),
            )
            .await;
        }
        Transaction::Auction(value) => {
            println!("\n{:?}", value);
            let mut auction_house = shared_auction_house.lock().await;
            let signed_content = digest(
                value.item_name.clone()
                    + &value.starting_bid.to_string()
                    + &value.user_id.to_string(),
            );
            match validate_tx_integrity(&signed_content, &value.user_id, value.signature.clone())
                .await
            {
                Ok(true) => {
                    auction_house.add_auction(value);
                }
                Ok(false) => println!("signature is not valid"),
                Err(e) => println!("{:?}", e),
            }
        }
    }
}
pub async fn validate_tx_integrity(
    signed_content: &String,
    uid: &String,
    sig_string: String,
) -> Result<bool, Box<dyn std::error::Error>> {
    // get puiblic key from uid hex value
    let public_key = VerifyingKey::from_sec1_bytes(&hex::decode(uid).unwrap())?;

    let sig: Signature = Signature::from_bytes(&GenericArray::clone_from_slice(
        &hex::decode(sig_string).unwrap(),
    ))?;

    // validate signature with the concat of parameters
    Ok(public_key.verify(signed_content.as_bytes(), &sig).is_ok())
}

pub async fn find_auction_to_bid(
    shared_auction_house: &mut Arc<Mutex<AuctionHouse>>,
    bid: &Bid,
    transaction: Transaction,
    requester_addr: String,
    routing_table: Vec<String>,
) {
    let mut auction_house = shared_auction_house.lock().await;

    if let Some(auction) = auction_house
        .clone()
        .auctions
        .iter()
        .find(|auction| auction.signature == bid.auction_signature)
    {
        let signed_content =
            digest(bid.auction_signature.clone() + &bid.bidder + &bid.amount.to_string());

        let mut last_highest_bid = 0.0;
        if !auction.clone().bids.is_empty() {
            last_highest_bid = auction.clone().bids[auction.clone().bids.len() - 1].amount;
        }

        if &auction.clone().end_time > &Utc::now() && last_highest_bid < bid.amount {
            match validate_tx_integrity(&signed_content, &bid.bidder, bid.signature.clone()).await {
                Ok(true) => {
                    let target_auction_position = auction_house
                        .auctions
                        .iter()
                        .position(|i| i.signature == auction.signature)
                        .unwrap();

                    auction_house.auctions[target_auction_position]
                        .bids
                        .push(bid.clone());

                    //Bid is valid send notification to client
                    if !auction_house.auctions[target_auction_position]
                        .subscribers
                        .contains(&requester_addr)
                    {
                        println!(
                            "New subscriber {} to auction: {}\n",
                            requester_addr.clone(),
                            auction_house.auctions[target_auction_position].signature
                        );
                        auction_house.auctions[target_auction_position]
                            .subscribers
                            .push(requester_addr.clone());
                    }
                    for peer in auction_house.clone().auctions[target_auction_position]
                        .subscribers
                        .clone()
                    {
                        if peer != requester_addr.clone() {
                            println!("sending notification to {}", peer.clone());
                            tokio::task::spawn(send_notification(peer.clone(), bid.clone()));
                        }
                    }
                }

                Ok(false) => println!("Error integrity signature is not valid"),
                Err(e) => println!("{:?}", e),
            }
        }
    } else {
        println!("Auction not present, sending to peers\n");
        //
        let dest_ip = routing_table
            .get(0)
            .unwrap()
            .clone()
            .to_string()
            .split(":")
            .next()
            .unwrap()
            .to_owned();

        match send_transaction(transaction, &dest_ip, requester_addr).await {
            Ok(result) => {
                println!(
                    "Dispaching Transaction to peer:{:?} {:?} ",
                    &dest_ip, result
                );
            }
            Err(e) => {
                println!("error {}", e);
            }
        }
    }
}
