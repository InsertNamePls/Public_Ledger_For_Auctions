use tokio::net::TcpStream;

use tokio::io::AsyncWriteExt;

use crate::auction::Auction;
use crate::transactions::Transaction;
pub async fn send_transaction(data: Auction, dest_addr: &str) {
    let data_str = serde_json::to_string(&data).unwrap();
    if let Ok(mut stream) = TcpStream::connect(format!("{}:3000", dest_addr)).await {
        if let Err(e) = stream.write_all(data_str.as_bytes()).await {
            eprintln!("error requesting data: {}", e);
        }
    }
}
