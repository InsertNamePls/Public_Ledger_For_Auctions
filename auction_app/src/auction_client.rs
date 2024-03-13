use tokio::net::{TcpListener, TcpStream};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
pub async fn send_transaction(data: String, dest_addr: &str) {
    //let block_serialized = serde_json::to_string(&data).unwrap();
    if let Ok(mut stream) = TcpStream::connect(format!("{}:3000", dest_addr)).await {
        if let Err(e) = stream.write_all(data.as_bytes()).await {
            eprintln!("error requesting data: {}", e);
        }
        println!("{:?}", stream);
        //     let mut buffer = [0; 1024];
        //     match stream.read(&mut buffer).await {
        //         Ok(n) => {
        //             let result = String::from_utf8_lossy(&buffer[..n]);
        //             println!("{}", result);
        //             if result == "valid" {
        //                 true
        //             } else {
        //                 false
        //             }
        //         }
        //         Err(e) => {
        //             eprintln!("error reading from server {}", e);
        //             false
        //         }
        //     }
        // } else {
        //     eprintln!("error conneting to server");
        //     false
        // }
    }
}

// #[tokio::main]
// async fn main() {
//     send_transaction("hello", "127.0.0.1").await;
// }
