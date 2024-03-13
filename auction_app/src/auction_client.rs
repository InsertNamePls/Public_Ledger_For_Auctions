use tokio::net::TcpStream;

use tokio::io::AsyncWriteExt;
pub async fn send_transaction(data: String, dest_addr: &str) {
    if let Ok(mut stream) = TcpStream::connect(format!("{}:3000", dest_addr)).await {
        if let Err(e) = stream.write_all(data.as_bytes()).await {
            eprintln!("error requesting data: {}", e);
        }
    }
}
