use rand::RngCore;
use rand::{distributions::Alphanumeric, Rng};
use rand_distr::{Poisson, Distribution};
use tonic::{Request, Code, Status};
use std::time::Duration;
use tokio::time::sleep;
use bytes::Bytes;
use crate::kademlia::kademlia_client::KademliaClient;
use crate::kademlia::{PingRequest, StoreRequest, FindNodeRequest, FindNodeResponse, StoreResponse};

pub async fn run_client(target: &str, command: &str) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = format!("http://{}", target);
    let mut client = KademliaClient::connect(endpoint).await?;

    match command {
        "ping" => {
            println!("Sending Ping Request");
            let request = Request::new(PingRequest {
                node_address: "client_node_id".into(), // Example node ID, adjust as needed
            });
            let response = client.ping(request).await?;
            println!("Received ping response: {:?}", response.into_inner());
        },
        "store" => {
                println!("Sending Store request");
                let key = generate_bytes(20).to_vec();
                let value = generate_bytes(10).to_vec();
                let store_request = StoreRequest { 
                    key, 
                    value 
                };

                let request = Request::new(store_request);
                let response = client.store(request).await?;
                println!("Store response: {:?}", response.into_inner());
        },
        _ => {
            println!("Unsupported command '{}'", command);
            return Ok(());
        }
    }

    Ok(())
}

/// Generates a random sequence of bytes of a given length
fn generate_bytes(len: usize) -> Bytes {
    let mut rng = rand::thread_rng();
    let mut id = vec![0u8; len]; // 160 bits
    rng.fill_bytes(&mut id);
    Bytes::from(id)
}
