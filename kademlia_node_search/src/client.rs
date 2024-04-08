use rand_distr::{Poisson, Distribution};
use tonic::{Request};
use std::time::Duration;
use tokio::time::sleep;
use crate::kademlia::{kademlia_client::KademliaClient, PingRequest};

pub async fn run_client(target: &str, command: &str) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = format!("http://{}", target);
    let mut client = KademliaClient::connect(endpoint).await?;

    // Lambda parameter for Poisson distribution: average number of events in an interval
    let lambda = 0.5; // Mean rate of requests per second, adjust as needed
    let poisson = Poisson::new(lambda).unwrap();

    match command {
        "ping" => {
            loop {
                // Generate the next interval
                let interval = poisson.sample(&mut rand::thread_rng());
                println!("Waiting for {:?} seconds before sending the next ping", interval);

                // Sleep for the interval determined by the Poisson distribution
                sleep(Duration::from_secs(interval as u64)).await;

                // Prepare and send a Ping request
                let request = Request::new(PingRequest {
                    source_node_id: "client_node_id".into(), // Example node ID, adjust as needed
                });

                match client.ping(request).await {
                    Ok(response) => println!("Received response: {:?}", response.into_inner()),
                    Err(e) => println!("Error: {:?}", e),
                }
            }
        },
        _ => {
            println!("Unsupported command '{}'", command);
            return Ok(());
        },
    }
}

