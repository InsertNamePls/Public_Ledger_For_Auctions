mod routing_table;
mod request_handler;
pub mod crypto;
pub mod client;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use rand_distr::{Distribution, Uniform};
use tokio::sync::Mutex;
use bytes::Bytes;
use routing_table::RoutingTable;
use request_handler::RequestHandler;
use crypto::Crypto;
use client::Client;
use tonic::transport::{Server};
use tonic::{Request, Response, Status};
use crate::kademlia::kademlia_server::{Kademlia, KademliaServer};
use crate::kademlia::{PingRequest, PingResponse, StoreRequest, StoreResponse, FindNodeRequest, FindNodeResponse, FindValueRequest, FindValueResponse};
use tokio::time::{Duration};
use self::routing_table::NodeInfo;
use rand::SeedableRng;
use colored::*;
use ring::{signature};
use ring::digest::{digest, SHA256};
use ring::signature::KeyPair;
//Config Constants
use crate::config::{C1, LOG_INTERVAL, REFRESH_TIMER_LOWER, REFRESH_TIMER_UPPER, PING_TIMER_LOWER, PING_TIMER_UPPER, N};

pub struct Node {
    pub keypair: signature::Ed25519KeyPair,
    pub id: Bytes,
    pub addr: SocketAddr,
    pub storage: Mutex<HashMap<Bytes, Bytes>>,
    pub routing_table: Mutex<RoutingTable>,
    pub crypto: Crypto,
    pub client: Client,
}

impl Node {
    pub async fn new(addr: SocketAddr, bootstrap_addr: Option<&str>) -> Result<Arc<Mutex<Self>>, Box<dyn std::error::Error + Send + Sync>> {
        let (keypair, node_id, duration, attempts) = Self::generate_id().await?;
        let routing_table = Mutex::new(RoutingTable::new(node_id.clone()));

        // Create a node instance within an Arc<Mutex<>> wrapper
        let node = Arc::new(Mutex::new(Node {
            keypair,
            id: node_id,
            addr,
            storage: Mutex::new(HashMap::new()),
            routing_table,
            crypto: Crypto::new(),
            client: Client::new(),
        }));

        // Print out the generated node ID
        println!("Generated Node ID: {}", hex::encode(&node.lock().await.id));
        println!("Time taken to generate Node ID: {:.2?}", duration);
        println!("Number of attempts: {}", attempts);

        // Fetch the bootstrap node's routing table if provided
        if let Some(addr) = bootstrap_addr {
            println!("{}", format!("Fetching routing table from bootstrap node: {}", addr).blue());
            node.lock().await.fetch_routing_table(addr).await?;
        }

        node.lock().await.routing_table.lock().await.print_table();

        Ok(node)
    }
    
    async fn generate_id() -> Result<(signature::Ed25519KeyPair, Bytes, Duration, u64), Box<dyn std::error::Error + Send + Sync>> {
        let c1 = C1; // Example difficulty level: number of leading zero bits
        let start_time = Instant::now();
        let mut attempts = 0;
        let attempt_log_interval = LOG_INTERVAL; // Print status every 10,000 attempts

        println!("Generating node ID with {} leading zero bits", c1);

        loop {
            attempts += 1;
            let keypair = Crypto::create_keypair()?;
            let public_key_hash = digest(&SHA256, keypair.public_key().as_ref());

            let node_id = Bytes::from(public_key_hash.as_ref().to_vec());

            if attempts % attempt_log_interval == 0 {
                let elapsed = start_time.elapsed();
                println!("Attempts: {}, Elapsed time: {:.2?} seconds", attempts, elapsed);
            }

            // Check if the first `c1` bits are zero (this implementation only works if c1 < 16)
            let valid = if c1 <= 8 {
                node_id[0] >> (8 - c1) == 0
            } else {
                node_id[0] == 0 && node_id[1] >> (16 - c1) == 0
            };

            if valid {
                let duration = start_time.elapsed();
                return Ok((keypair, node_id, duration, attempts));
            }
        }
    }

    async fn fetch_routing_table(&self, target_addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ping_request = self.client.create_ping_request(&self.keypair, self.id.to_vec(), self.addr.to_string());
        match self.client.send_ping_request(ping_request, target_addr.to_string()).await {
            Ok(ping_response) => {
                println!("{}", format!("Received ping response: {:?}", ping_response).green());

                let find_node_request = self.client.create_find_node_request(
                    &self.keypair,
                    self.id.to_vec(),
                    self.addr.to_string(),
                    ping_response.node_id.to_vec()
                );

                match self.client.send_find_node_request(find_node_request, target_addr.to_string()).await {
                    Ok(find_node_response) => {
                        println!("{}", format!("Received find_node response: {:?}", find_node_response).green());
                        self.update_routing_table(RoutingTable::from_proto_nodes(find_node_response.nodes)).await;
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("Failed to send find_node request: {}", e);
                        self.routing_table.lock().await.remove_node(&Bytes::from(target_addr.as_bytes().to_vec()));
                        Err(Box::new(e))
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to send ping request: {}", e);
                self.routing_table.lock().await.remove_node(&Bytes::from(target_addr.as_bytes().to_vec()));
                Err(Box::new(e))
            }
        }
    }

    async fn update_routing_table(&self, nodes: Vec<NodeInfo>) {
        let mut routing_table = self.routing_table.lock().await;
        let mut _counter: i64 = 0;
        for new_node in nodes {
            if self.id != new_node.id && !routing_table.contains(&new_node.id) {
                _counter += 1;
                routing_table.add_node(new_node, &self.id);
            }
        }
        println!("Added {} new nodes to the routing table", _counter);
    }

    async fn refresh_routing_table(node: Arc<Mutex<Node>>) {
        let interval_range = Uniform::from(REFRESH_TIMER_LOWER..REFRESH_TIMER_UPPER);
        let mut rng = rand_chacha::ChaChaRng::from_entropy(); // RNG should be outside the loop to preserve state and performance
    
        loop {
            let sleep_time = interval_range.sample(&mut rng);
            println!("{}", format!("Fetching routing table in {} seconds", sleep_time).cyan());
            tokio::time::sleep(Duration::from_secs(sleep_time)).await;
    
            // Lock only when needed and scope the lock to minimize blocking
            let maybe_node_info = {
                let node_lock = node.lock().await;
                let routing_table = node_lock.routing_table.lock().await;
                routing_table.random_node().cloned() // Clone the data to use outside the lock
            };
    
            if let Some(node_info) = maybe_node_info {
                println!("{}", format!("Refreshing routing table from node: {:?}", node_info.addr).cyan());
                // Fetch the routing table outside of the node locks
                let result = {
                    let node_lock = node.lock().await;
                    node_lock.fetch_routing_table(&node_info.addr.to_string()).await
                };
    
                match result {
                    Ok(_) => println!("{}", format!("Routing table refreshed successfully from {}", node_info.addr).green()),
                    Err(e) => {
                        eprintln!("{}", format!("Failed to refresh routing table from {}: {}", node_info.addr, e).red());
                        eprintln!("{}", format!("Removing node {} from routing table", node_info.addr).yellow());
                        node.lock().await.routing_table.lock().await.remove_node(&node_info.id);
                    },
                }
            }
            else {
                eprintln!("{}", "No nodes available to refresh the routing table".yellow());
            }
            node.lock().await.routing_table.lock().await.print_table();
        }
    }

    async fn check_nodes_alive(node: Arc<Mutex<Node>>) {
        let interval_range = Uniform::from(PING_TIMER_LOWER..PING_TIMER_UPPER);
        let mut rng = rand_chacha::ChaChaRng::from_entropy(); // RNG should be outside the loop to preserve state and performance
    
        loop {
            let sleep_time = interval_range.sample(&mut rng);
            println!("{}", format!("Checking node liveness in {} seconds", sleep_time).cyan());
            tokio::time::sleep(Duration::from_secs(sleep_time)).await;
    
            // Lock only when needed and scope the lock to minimize blocking
            let node_infos: Vec<NodeInfo> = {
                let node_lock = node.lock().await;
                let routing_table = node_lock.routing_table.lock().await;
                routing_table.random_nodes(N) // Get N random nodes
            };
    
            for node_info in node_infos {
                println!("{}", format!("Pinging node: {:?}", node_info.addr).cyan());
                // Send ping request outside of the node locks
                let result = {
                    let node_lock = node.lock().await;
                    let ping_request = node_lock.client.create_ping_request(&node_lock.keypair, node_lock.id.to_vec(), node_lock.addr.to_string());
                    node_lock.client.send_ping_request(ping_request, node_info.addr.to_string()).await
                };
    
                match result {
                    Ok(_) => println!("{}", format!("Node {} is alive", node_info.addr).green()),
                    Err(e) => {
                        eprintln!("{}", format!("Node {} is unreachable: {}", node_info.addr, e).red());
                        eprintln!("{}", format!("Removing node {} from routing table", node_info.addr).yellow());
                        node.lock().await.routing_table.lock().await.remove_node(&node_info.id);
                    },
                }
            }
        }
    }
}

#[tonic::async_trait]
impl Kademlia for Arc<Mutex<Node>> {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let node = Arc::clone(self);
        println!("{}", format!("Received ping request: {:?}", request).blue());

        RequestHandler::handle_ping(node, request).await
    }

    async fn store(&self, request: Request<StoreRequest>) -> Result<Response<StoreResponse>, Status> {
        let node = Arc::clone(self);
        println!("{}", format!("Received store request: {:?}", request).blue());

        RequestHandler::handle_store(node, request).await
    }

    async fn find_node(&self, request: Request<FindNodeRequest>) -> Result<Response<FindNodeResponse>, Status> {
        let node = Arc::clone(self);
        println!("{}", format!("Received find_node request: {:?}", request).blue());

        RequestHandler::handle_find_node(node, request).await
    }

    async fn find_value(&self, request: Request<FindValueRequest>) -> Result<Response<FindValueResponse>, Status> {
        let node = Arc::clone(self);
        println!("{}", format!("Received find_value request: {:?}", request).blue());

        RequestHandler::handle_find_value(node, request).await
    }
}

pub async fn run_server(addr: &str, bootstrap_addr: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = addr.parse::<SocketAddr>()?;
    let node = Node::new(addr, bootstrap_addr.as_deref()).await?;

    let node_clone_for_server = Arc::clone(&node);
    tokio::spawn(async move {
        Node::refresh_routing_table(node_clone_for_server.clone()).await;
    });

    let node_clone_for_checking = Arc::clone(&node);
    tokio::spawn(async move {
        Node::check_nodes_alive(node_clone_for_checking.clone()).await;
    });

    println!("{}", format!("Server listening on {}", addr).green());
    Server::builder()
        .add_service(KademliaServer::new(Arc::clone(&node)))
        .serve(addr)
        .await?;

    Ok(())
}