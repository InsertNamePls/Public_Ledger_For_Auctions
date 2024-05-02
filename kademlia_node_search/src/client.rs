use rand::RngCore;
use rand::{distributions::Alphanumeric, Rng};
use rand_distr::{Poisson, Distribution};
use tonic::{Request, Code, Status};
use std::time::Duration;
use tokio::time::sleep;
use bytes::Bytes;
use crate::kademlia::kademlia_client::KademliaClient;
use crate::kademlia::{PingRequest, StoreRequest, FindNodeRequest, FindNodeResponse, StoreResponse, FindValueRequest, FindValueResponse};
use std::env;

pub async fn run_client(target: &str, command: &str) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = format!("http://{}", target);
    let mut client = KademliaClient::connect(endpoint).await?;

    // Collect user input for the key
    println!("Enter a key (text) for the operation:");
    let mut user_input = String::new();
    std::io::stdin().read_line(&mut user_input)?;
    let user_input_bytes = user_input.trim().to_string().into_bytes();
    let key = Bytes::from(user_input_bytes);

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
            println!("Sending Store request with key: {}", user_input.trim());
            let value = generate_bytes(10).to_vec();  // Generate a random value
            let store_request = StoreRequest { 
                key: key.to_vec(), 
                value 
            };

            let request = Request::new(store_request);
            let response = client.store(request).await?;
            println!("Store response: {:?}", response.into_inner());
        },
        "find_value" => {
            println!("Sending Find Value request for key: {}", user_input.trim());
            let find_value_request = FindValueRequest { 
                key: key.to_vec(),
            };

            let request = Request::new(find_value_request);
            let response = client.find_value(request).await?;
            match response.into_inner() {
                FindValueResponse { value, nodes } => {
                    if !value.is_empty() {
                        println!("Value found: {:?}", value);
                    } else {
                        println!("Value not found, closest nodes: {:?}", nodes);
                    }
                }
            }
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
