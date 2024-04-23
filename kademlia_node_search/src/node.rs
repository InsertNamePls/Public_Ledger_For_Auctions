mod routing_table;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
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


//Upper limit for random routing table refresh
const REFRESH_TIMER_UPPER: u64 = 20;
//Lower limit for random routing table refresh
const REFRESH_TIMER_LOWER: u64 = 5;
//Timeout for each request
const TIMEOUT_TIMER: u64 = 3;
//Maximum number of attempts for each request
const TIMEOUT_MAX_ATTEMPTS: u64 = 3;

pub struct Node {
    id: Bytes,
    storage: Mutex<HashMap<Bytes, Bytes>>,
    routing_table: Mutex<RoutingTable>,
}

impl Node {
    pub async fn new(addr: SocketAddr, bootstrap_addr: Option<&str>) -> Result<Arc<Mutex<Self>>, Box<dyn std::error::Error>> {
        let node_id = Self::generate_id().await;
        let routing_table = Mutex::new(RoutingTable::new(node_id.clone()));

        println!("{:?}", format!("Generated node ID: {:?}", node_id).green().bold());

        // Create a node instance within an Arc<Mutex<>> wrapper
        let node = Arc::new(Mutex::new(Node {
            id: node_id.clone(),
            storage: Mutex::new(HashMap::new()),
            routing_table,
        }));

        // Add self to routing table
        {
            let node_lock = node.lock().await;
            let mut routing_table = node_lock.routing_table.lock().await;
            routing_table.add_node(NodeInfo {
                id: node_id.clone(),
                addr: addr,
            }, &node_id);
        }

        // Fetch the bootstrap node's routing table if provided
        if let Some(addr) = bootstrap_addr {
            println!("{}", format!("Fetching routing table from bootstrap node: {}", addr).blue());
            node.lock().await.fetch_routing_table(addr).await?;
        }

        node.lock().await.routing_table.lock().await.print_table();

        Ok(node)
    }

    async fn generate_id() -> Bytes {
        let mut rng = rand::thread_rng();
        let mut id = vec![0u8; 20]; // 160 bits
        rng.fill_bytes(&mut id);
        Bytes::from(id)
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
            if !routing_table.contains(&new_node.id) {
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
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let node = self.lock().await;
        println!("{}", format!("Received ping request: {:?}", request).blue());
        let response = PingResponse {
            is_online: true,
            node_id: node.id.clone().to_vec(), // Ensure proper Bytes to Vec<u8> conversion
        };
        Ok(Response::new(response))
    }

    async fn store(&self, request: Request<StoreRequest>) -> Result<Response<StoreResponse>, Status> {
        let node = self.lock().await;
        println!("{}", format!("Received store request: {:?}", request).blue());
        let store_request = request.into_inner();
        let key = Bytes::from(store_request.key);
        let value = Bytes::from(store_request.value);
    
        let mut storage = node.storage.lock().await;
        // Insert the key-value pair into the local storage.
        storage.insert(key, value);
    
        // Acknowledge that the store operation succeeded.
        Ok(Response::new(StoreResponse { success: true }))
    }

    async fn find_node(&self, request: Request<FindNodeRequest>) -> Result<Response<FindNodeResponse>, Status> {
        let node = self.lock().await;
        println!("{}", format!("Received find_node request: {:?}", request).blue());
        let target_id = Bytes::from(request.into_inner().target_node_id);
    
        // Retrieve the closest nodes from the routing table
        let closest_nodes = {
            let routing_table = node.routing_table.lock().await;
            routing_table.find_closest(&target_id, &node.id)
        };
    
        // Log the found nodes and prepare the response
        let proto_nodes = closest_nodes.iter().map(|node_info| ProtoNodeInfo {
            id: node_info.id.clone().to_vec(),
            address: node_info.addr.to_string(),
        }).collect::<Vec<_>>();

        Ok(Response::new(FindNodeResponse { nodes: proto_nodes }))
    }

    async fn find_value(&self, request: Request<FindValueRequest>) -> Result<Response<FindValueResponse>, Status> {
        let node: tokio::sync::MutexGuard<'_, Node> = self.lock().await;
        println!("{}", format!("Received find_value request: {:?}", request).blue());
        let find_value_request = request.into_inner();
        let key = Bytes::from(find_value_request.key);
        println!("{}", format!("Key requested: {:?}", key).yellow());
    
        unimplemented!();
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
