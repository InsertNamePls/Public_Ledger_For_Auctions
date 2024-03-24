use super::auction::*;
use crate::auction_client::send_transaction;
use chrono::Utc;
use std::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub async fn retrieve_auction_house() {
    let listener = TcpListener::bind("0.0.0.0:3004").await.unwrap();

    while let Ok((mut socket, _)) = listener.accept().await {
        let local_auction_house =
            fs::read_to_string("auction_data.json").expect("Unable to read file");

        let mut buffer = [0; 2048];
        match socket.read(&mut buffer).await {
            Ok(0) => break,
            Ok(_n) => {
                if let Err(e) = socket.write_all(local_auction_house.as_bytes()).await {
                    eprintln!("error sending data: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Error reading from buffer: {}", e);
            }
        }
    }
}

pub async fn auction_server(dest_ip: String) {
    let listener = TcpListener::bind(format!("{}:3000", "0.0.0.0".to_string()))
        .await
        .unwrap();
    while let Ok((mut socket, _)) = listener.accept().await {
        let mut buffer = [0; 2048];
        match socket.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                let request = String::from_utf8_lossy(&buffer[..n]);
                let transaction: Transaction = serde_json::from_str(&request).unwrap();
                let data = fs::read_to_string("auction_data.json").expect("Unable to read file");
                let mut auction_house: AuctionHouse =
                    serde_json::from_str(&data).expect("Failed to deserialize JSON");
                match transaction {
                    Transaction::Bid(ref value) => {
                        println!("\n{:?}", value);

                        find_auction_to_bid(
                            auction_house.clone(),
                            &value,
                            &transaction.clone(),
                            socket,
                            dest_ip.clone(),
                        )
                        .await;
                    }
                    Transaction::Auction(value) => {
                        println!("\n{:?}", auction_house);

                        auction_house.add_auction(value);
                        let serialized = serde_json::to_string_pretty(&auction_house).unwrap();
                        fs::write("auction_data.json", serialized);
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
pub async fn find_auction_to_bid(
    mut auction_house: AuctionHouse,
    bid: &Bid,
    transaction: &Transaction,
    socket: TcpStream,
    dest_ip: String,
) {
    if let Some(auction) = auction_house
        .auctions
        .iter()
        .find(|auction| auction.auction_id == bid.auction_id)
    {
        auction_house =
            bid_handler(auction.clone(), auction_house.clone(), bid.clone(), socket).await;
        let serialized = serde_json::to_string_pretty(&auction_house).unwrap();
        fs::write("auction_data.json", serialized);
    } else {
        println!("Auction not present, sending to other peers");
        //
        send_transaction(transaction, dest_ip).await;
    }
}

pub async fn bid_handler(
    mut target_auction: Auction,
    mut auction_house: AuctionHouse,
    bid: Bid,
    mut socket: TcpStream,
) -> AuctionHouse {
    let mut last_highest_bid = 0.0;
    if !target_auction.bids.is_empty() {
        last_highest_bid = target_auction.bids[target_auction.bids.len() - 1].amount;
    }

    if &target_auction.end_time > &Utc::now() && last_highest_bid < bid.amount {
        let target_auction_position = auction_house
            .auctions
            .iter()
            .position(|auction| auction.auction_id == target_auction.auction_id)
            .unwrap();

        auction_house.auctions[target_auction_position]
            .bids
            .push(bid);

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
