use crate::auction_app::{save_auction_data, AuctionHouse, Transaction};
use crate::helper::bid_handler;
use local_ip_address::local_ip;
use std::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub async fn auction_server() {
    let my_local_ip = local_ip().unwrap();
    let listener = TcpListener::bind(format!("{}:3000", "0.0.0.0".to_string()))
        .await
        .unwrap();
    //loop {
    while let Ok((mut socket, _)) = listener.accept().await {
        let mut buffer = [0; 2048];
        match socket.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                let request = String::from_utf8_lossy(&buffer[..n]);
                let m: Transaction = serde_json::from_str(&request).unwrap();
                let data = fs::read_to_string("auction_data.json").expect("Unable to read file");
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
                        let serialized = serde_json::to_string_pretty(&auction_house).unwrap();
                        fs::write("auction_data.json", serialized);
                    }
                    Transaction::Auction(value) => {
                        println!("\n{:?}", auction_house);

                        auction_house.add_auction(value, (auction_house.auctions.len() as u32));
                        let serialized = serde_json::to_string_pretty(&auction_house).unwrap();
                        fs::write("auction_data.json", serialized);
                        // save_auction_data(&auction_house).expect("erro saving auction file");
                    }
                }
                println!("{:?}", auction_house);
            }
            Err(e) => {
                eprintln!("Error reading from buffer: {}", e);
            }
        }
    }
    //}
}
//
pub async fn retrieve_blockchain() {
    let listener = TcpListener::bind("0.0.0.0:3002").await.unwrap();

    while let Ok((mut socket, _)) = listener.accept().await {
        let local_blockchain = fs::read_to_string("blockchain.json").expect("Unable to read file");

        let mut buffer = [0; 2048];
        match socket.read(&mut buffer).await {
            Ok(0) => break,
            Ok(_n) => {
                println!("{:?}", local_blockchain);
                if let Err(e) = socket.write_all(local_blockchain.as_bytes()).await {
                    eprintln!("error sending data: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Error reading from buffer: {}", e);
            }
        }
    }
}

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
