use crate::auction::{save_auction_data, Auction, AuctionHouse, Transaction};
use std::process::exit;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
pub async fn send_transaction(data: Transaction, dest_addr: &str) {
    let data_str = serde_json::to_string(&data).unwrap();
    if let Ok(mut stream) = TcpStream::connect(format!("{}:3000", dest_addr)).await {
        if let Err(e) = stream.write_all(data_str.as_bytes()).await {
            eprintln!("error requesting data: {}", e);
        }
        let mut buffer = [0; 2048];
        match stream.read(&mut buffer).await {
            Ok(n) => {
                let result = String::from_utf8_lossy(&buffer[..n]);
                println!("{}", result)
            }
            Err(e) => {
                eprintln!("error reading from server {}", e);
            }
        }
    }
}

pub async fn request_auction_house(dest_addr: &str) {
    if let Ok(mut stream) = TcpStream::connect(format!("{}:3004", dest_addr)).await {
        if let Err(e) = stream.write_all("get_auction".as_bytes()).await {
            eprintln!("error requesting data: {}", e);
        }
        let mut buffer = [0; 2048];
        match stream.read(&mut buffer).await {
            Ok(n) => {
                let result = String::from_utf8_lossy(&buffer[..n]);
                let auctions: AuctionHouse = serde_json::from_str(&result).unwrap();
                save_auction_data(&auctions);
            }
            Err(e) => {
                eprintln!("error reading from server {}", e);
            }
        }
    }
}
