use crate::blockchain::{validate_block, Block, Blockchain};
use std::process::exit;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use async_std::fs;
const DIFICULTY: usize = 4;

pub async fn handle_connection(mut socket: TcpStream, data: String) {
    let mut buffer = [0; 1024];
    match socket.read(&mut buffer).await {
        Ok(n) => {
            //process Response
            println!("{:?}", data);
            if let Err(e) = socket.write_all(data.as_bytes()).await {
                eprintln!("error sending data: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Error reading from buffer: {}", e)
        }
    }
}
pub async fn connection_handler(mut socket: TcpStream) -> (Block, TcpStream) {
    let mut buffer = [0; 1024];
    match socket.read(&mut buffer).await {
        //Ok(0) => (),
        Ok(n) => {
            //convert slive of bytes to string
            let request = String::from_utf8_lossy(&buffer[..n]);
            println!("{}", request);
            let new_block: Block =
                serde_json::from_str(&request).expect("Failed to deserialize JSON");

            (new_block, socket)
        }
        Err(e) => {
            eprintln!("Error reading from buffer: {}", e);
            exit(0);
        }
    }
}
pub async fn validator(new_block: &Block, previous_block: &Block) -> bool {
    let validation_result = validate_block(new_block, previous_block, DIFICULTY);
    validation_result
}

pub async fn server_block_validate() {
    let listener = TcpListener::bind("0.0.0.0:3003").await.unwrap();

    while let Ok((socket, _)) = listener.accept().await {
        let data = fs::read_to_string("./blockchain.txt")
            .await
            .expect("Unable to read file");
        let mut local_blockchain: Blockchain =
            serde_json::from_str(&data).expect("Failed to deserialize JSON");
        let previous_block_str = local_blockchain.blocks.last().unwrap();
        let previous_block: Block =
            serde_json::from_str(previous_block_str).expect("Failed to deserialize JSON");
        //let data_share = Arc::new(Mutex::new(previous_block_str.to_string()));
        //
        println!("server block validate -> {:?}", local_blockchain);

        //let data_share_clone = Arc::clone(&data_share);

        //tokio::spawn(handle_validate_connection(socket, data_share_clone));

        let (new_block, mut result_socket) = connection_handler(socket).await;

        if validator(&new_block, &previous_block).await {
            result_socket.write_all("valid".as_bytes()).await;

            let new_block_str = serde_json::to_string(&new_block).unwrap();

            local_blockchain.add_block(&new_block_str);
            let bch_str = serde_json::to_string(&local_blockchain).unwrap();
            println!("save locally {}", bch_str);
            save_blockchain_locally(&local_blockchain).await;
        } else {
            result_socket.write_all("invalid".as_bytes()).await;
        }
        //if handle_validate_connection(socket, previous_block_str).await {
        println!("Blockchain {:?}", local_blockchain);
    }
}
pub async fn network_validate_block(data: &Block, dest_addr: &String) -> bool {
    let block_serialized = serde_json::to_string(&data).unwrap();
    if let Ok(mut stream) = TcpStream::connect(format!("{}:3003", dest_addr)).await {
        if let Err(e) = stream.write_all(block_serialized.as_bytes()).await {
            eprintln!("error requesting data: {}", e);
        }
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer).await {
            Ok(n) => {
                let result = String::from_utf8_lossy(&buffer[..n]);
                println!("{}", result);
                if result == "valid" {
                    true
                } else {
                    false
                }
            }
            Err(e) => {
                eprintln!("error reading from server {}", e);
                false
            }
        }
    } else {
        eprintln!("error conneting to server");
        false
    }
}
pub async fn save_blockchain_locally(blockchain: &Blockchain) {
    let chain_serialized = serde_json::to_string(&blockchain).unwrap();
    fs::write("./blockchain.txt", chain_serialized)
        .await
        .expect("Unable to write file");
}
