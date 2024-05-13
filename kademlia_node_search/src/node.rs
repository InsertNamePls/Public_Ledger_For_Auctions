mod routing_table;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use rand_distr::{Distribution, Uniform};
use tokio::sync::Mutex;
use bytes::Bytes;
use routing_table::RoutingTable;
use tonic::transport::{Endpoint, Server};
use tonic::{Request, Response, Status};
use crate::kademlia::kademlia_client::KademliaClient;
use crate::kademlia::kademlia_server::{Kademlia, KademliaServer};
use crate::kademlia::{NodeInfo as ProtoNodeInfo,PingRequest, PingResponse, StoreRequest, StoreResponse, FindNodeRequest, FindNodeResponse, FindValueRequest, FindValueResponse};
use crate::node;
use rand::{thread_rng, RngCore};
use tokio::time::{self, Duration,timeout};
use self::routing_table::NodeInfo;
use rand::SeedableRng;
use colored::*; 
use hex::ToHex; 
use ring::{rand as ring_rand, signature};
use ring::digest::{digest, SHA256};
use ring::rand::SecureRandom;
use ring::signature::KeyPair;

use sha2::{Sha256, Digest};
use rand::rngs::OsRng;





//Upper limit for random routing table refresh
const REFRESH_TIMER_UPPER: u64 = 20;
//Lower limit for random routing table refresh
const REFRESH_TIMER_LOWER: u64 = 5;
//Timeout for each request
const TIMEOUT_TIMER: u64 = 3;
//Maximum number of attempts for each request
const TIMEOUT_MAX_ATTEMPTS: u64 = 3;
//Leading zero bits for node ID generation
const C1: u32 = 8;
pub struct Node {
    pub keypair: signature::Ed25519KeyPair,
    pub id: Bytes,
    pub addr: SocketAddr,
    pub storage: Mutex<HashMap<Bytes, Bytes>>,
    pub routing_table: Mutex<RoutingTable>,
}

impl Node {
    pub async fn new(addr: SocketAddr, bootstrap_addr: Option<&str>) -> Result<Arc<Mutex<Self>>, Box<dyn std::error::Error>> {
        let (keypair, node_id, duration) = Self::generate_id().await?;
        let routing_table = Mutex::new(RoutingTable::new(node_id.clone()));

        // Create a node instance within an Arc<Mutex<>> wrapper
        let node = Arc::new(Mutex::new(Node {
            keypair,
            id: node_id,
            addr,
            storage: Mutex::new(HashMap::new()),
            routing_table,
        }));


         // Print out the generated node ID
        println!("Generated Node ID: {}", hex::encode(&node.lock().await.id));
        println!("Time taken to generate Node ID: {:.2?}", duration);

        // Fetch the bootstrap node's routing table if provided
        if let Some(addr) = bootstrap_addr {
            println!("{}", format!("Fetching routing table from bootstrap node: {}", addr).blue());
            node.lock().await.fetch_routing_table(addr).await?;
        }

        node.lock().await.routing_table.lock().await.print_table();

        Ok(node)
    }

    async fn generate_id() -> Result<(signature::Ed25519KeyPair, Bytes, std::time::Duration), Box<dyn std::error::Error>> {
        let rng = ring_rand::SystemRandom::new();
        let c1 = C1; // difficulty level: number of leading zero bits
        let start_time = Instant::now();

        loop {
            let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)
                .map_err(|_| "Failed to generate pkcs8 bytes")?;
            let keypair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
                .map_err(|_| "Failed to create keypair from pkcs8 bytes")?;
            let public_key_hash = digest(&SHA256, keypair.public_key().as_ref());

            let node_id = Bytes::from(public_key_hash.as_ref().to_vec());
            if node_id[0].leading_zeros() >= c1 as u32 {
                let duration = start_time.elapsed();
                return Ok((keypair, node_id, duration));
            }
        }
    }

    async fn fetch_routing_table(&self, bootstrap_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let endpoint = Endpoint::from_shared(format!("http://{}", bootstrap_addr))?;
        let channel = endpoint.connect().await?;
        let mut client = KademliaClient::new(channel);
        
        for attempt in 0..TIMEOUT_MAX_ATTEMPTS {
            println!("{}", format!("Attempt {} to fetch routing table from {}", attempt + 1, bootstrap_addr).yellow());
            
            let ping_request = Request::new(PingRequest { node_address: bootstrap_addr.to_string() });
            match timeout(Duration::from_secs(TIMEOUT_TIMER), client.ping(ping_request)).await {
                Ok(Ok(response)) => {
                    let ping_response = response.into_inner();
                    println!("{}", format!("Received ping response: {:?}", ping_response).green());
                    let find_node_request = Request::new(FindNodeRequest {
                        requester_node_id: self.id.to_vec(),
                        requester_node_address: self.addr.to_string(),
                        target_node_id: ping_response.node_id,
                    });

                    match timeout(Duration::from_secs(TIMEOUT_TIMER), client.find_node(find_node_request)).await {
                        Ok(Ok(find_response)) => {
                            println!("{}", format!("Received find_node response: {:?}", find_response).green());
                            let response = find_response.into_inner();
                            self.update_routing_table(RoutingTable::from_proto_nodes(response.nodes)).await;
                            return Ok(());
                        },
                        Ok(Err(e)) => eprintln!("{}", format!("Failed to receive find_node response: {}", e).red()),
                        Err(_) => eprintln!("{}", "Timeout during find_node request".red()),
                    }
                },
                Ok(Err(e)) => eprintln!("{}", format!("Failed to receive ping response: {}", e).red()),
                Err(_) => eprintln!("{}", "Timeout during ping request".red()),
            }
        }
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to fetch routing table after 3 attempts")))
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
                    Err(e) => eprintln!("{}", format!("Failed to refresh routing table from {}: {}", node_info.addr, e).red()),
                }
            }
            else {
                eprintln!("{}", "No nodes available to refresh the routing table".yellow());
            }
            node.lock().await.routing_table.lock().await.print_table();
        }
    }
    

}

#[tonic::async_trait]
impl Kademlia for Arc<Mutex<Node>> {
    // Implement the gRPC service methods

    // The ping method is used to check if a node is online.
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let node = self.lock().await;
        println!("{}", format!("Received ping request: {:?}", request).blue());
        let response = PingResponse {
            is_online: true,
            node_id: node.id.clone().to_vec(), // Ensure proper Bytes to Vec<u8> conversion
        };
        Ok(Response::new(response))
    }

    // The store method is used to store a key-value pair across the entire network.
    async fn store(&self, request: Request<StoreRequest>) -> Result<Response<StoreResponse>, Status> {
        let node = self.lock().await;
        println!("{}", format!("Received store request: {:?}", request).blue());

        let store_request = request.into_inner();
        let key = Bytes::from(store_request.key.clone());  // Clone to use later for forwarding
        let value = Bytes::from(store_request.value.clone());

        // Access local storage to check for the key and possibly store it
        let mut storage = node.storage.lock().await;
        let should_forward = match storage.get(&key) {
            Some(existing_value) if existing_value == &value => {
                // If the key exists with the same value, do not forward
                false
            },
            _ => {
                // Insert the key-value pair into the local storage
                storage.insert(key.clone(), value.clone());
                let key_hex = format!("{:x}", key);
                let value_hex = format!("{:x}",value);
                println!("Added <key:value> to local storage: <{}:{}>",key_hex,value_hex);
                true  // Forward the request as this is new or updated information
            }
        };


        // Forward the store request to closest nodes only if it's new or updated information
        if should_forward {
            //print_storage(&node).await;
            let closest_nodes = {
                let routing_table = node.routing_table.lock().await;
                routing_table.find_closest(&key)
            };


            // Use a lightweight task spawning to handle the requests asynchronously
            for node_info in closest_nodes.iter() {
                let client_addr = node_info.addr.to_string();
                let forward_store_request = StoreRequest {
                    key: store_request.key.clone(),
                    value: store_request.value.clone(),
                };

                // Skip sending to self
                if client_addr != node.addr.to_string() {
                    tokio::spawn(async move {
                        if let Ok(mut client) = KademliaClient::connect(format!("http://{}", client_addr)).await {
                            let request: Request<StoreRequest> = Request::new(forward_store_request);
                            let _ = client.store(request).await;
                            // Note: Errors are ignored as we do not wait for response
                        }
                    });
                }
            }

        }

        // Acknowledge that the local store operation was initiated successfully.
        Ok(Response::new(StoreResponse { success: true }))
    }


    // The find_node method is used to find the K closest nodes to a target node ID.
    async fn find_node(&self, request: Request<FindNodeRequest>) -> Result<Response<FindNodeResponse>, Status> {
        let node = self.lock().await;
    
        // Extract all necessary data before moving `request`
        let req_inner = request.into_inner();
        let target_id = Bytes::from(req_inner.target_node_id.clone()); // Clone if necessary
        let requester_id = Bytes::from(req_inner.requester_node_id);
        let requester_address = req_inner.requester_node_address;
        
        // Log the incoming request
        println!("{}", format!("Received find_node request from [{}]: target_id {}, requester_id {}", 
                              requester_address, 
                              target_id.encode_hex::<String>(), 
                              requester_id.encode_hex::<String>()).blue());
    
        // Retrieve the closest nodes from the routing table
        let closest_nodes = {
            let routing_table = node.routing_table.lock().await;
            routing_table.find_closest(&target_id)
        };
    
        // Check if the requester's node info is already in the routing table
        let mut routing_table = node.routing_table.lock().await;
        if !routing_table.contains(&requester_id) {
            let requester_node_info = NodeInfo {
                id: requester_id,
                addr: requester_address.parse::<SocketAddr>().unwrap(), // Assumption: the address is valid and can be parsed
            };
            
            // Try adding the requester to the routing table
            routing_table.add_node(requester_node_info, &node.id);
            println!("{}", "Added requester's information to routing table.".green());
            println!("{}", "Updated routing table:".green());
            routing_table.print_table();
        }
    
        // Prepare the response with the found nodes
        let proto_nodes = closest_nodes.iter().map(|node_info| ProtoNodeInfo {
            id: node_info.id.clone().to_vec(),
            address: node_info.addr.to_string(),
        }).collect::<Vec<_>>();
    
        Ok(Response::new(FindNodeResponse { nodes: proto_nodes }))
    }


   // The find_value method is used to find the value associated with a key in the DHT.
    async fn find_value(&self, request: Request<FindValueRequest>) -> Result<Response<FindValueResponse>, Status> {
        let node = self.lock().await;
        println!("{}", format!("Received find_value request: {:?}", request).blue());
        let find_value_request = request.into_inner();
        let key = Bytes::from(find_value_request.key);
        println!("{}", format!("Key requested: {:?}", key).yellow());

        // First, check if the key is present in the local storage
        let storage = node.storage.lock().await;
        if let Some(value) = storage.get(&key) {
            // If the key is found, return the value
            return Ok(Response::new(FindValueResponse {
                value: value.clone().to_vec(),  // Assuming value is `Bytes` and needs to be converted to `Vec<u8>`
                nodes: vec![],  // No nodes need to be returned since the value was found
            }));
        }

        // If the key is not found locally, find the closest nodes
        let routing_table = node.routing_table.lock().await;
        let closest_nodes = routing_table.find_closest(&key);
        
        // Convert NodeInfo from internal structure to protobuf format
        let proto_nodes = closest_nodes.iter().map(|node_info| {
            crate::kademlia::NodeInfo {
                id: node_info.id.clone().to_vec(),
                address: node_info.addr.to_string(),
            }
        }).collect::<Vec<_>>();

        // Respond with the closest nodes if the value is not found locally
        Ok(Response::new(FindValueResponse {
            value: Vec::new(),  // No value found
            nodes: proto_nodes, // Closest nodes to the requested key
        }))
    }
}


pub async fn run_server(addr: &str, bootstrap_addr: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr.parse::<SocketAddr>()?;
    let node = Node::new(addr, bootstrap_addr.as_deref()).await?;

    let node_clone_for_server = Arc::clone(&node);
    tokio::spawn(Node::refresh_routing_table(node_clone_for_server));

    println!("{}", format!("Server listening on {}", addr).green());
    Server::builder()
        .add_service(KademliaServer::new(Arc::clone(&node)))
        .serve(addr)
        .await?;

    Ok(())
}
