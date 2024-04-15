mod routing_table;

use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::Mutex;
use bytes::Bytes;
use routing_table::RoutingTable;
use tonic::transport::{Endpoint, Server};
use tonic::{Request, Response, Status};
use crate::kademlia::kademlia_client::KademliaClient;
use crate::kademlia::kademlia_server::{Kademlia, KademliaServer};
use crate::kademlia::{NodeInfo as ProtoNodeInfo,PingRequest, PingResponse, StoreRequest, StoreResponse, FindNodeRequest, FindNodeResponse, FindValueRequest, FindValueResponse};
use rand::RngCore;



use self::routing_table::NodeInfo;

pub struct Node {
    id: Bytes,
    storage: Mutex<HashMap<Bytes, Bytes>>,
    routing_table: Mutex<RoutingTable>,
}

impl Node {
    pub async fn new(addr: SocketAddr, bootstrap_addr: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let node_id = Self::generate_id().await;
        let mut routing_table = RoutingTable::new();

        println!(">Generated Node ID: {:?}", node_id);

        // Add self to routing table
        let self_info = NodeInfo {
            id: node_id.clone(),
            addr,
        };
        routing_table.add_node(self_info, &node_id);

        let node = Node {
            id: node_id,
            storage: Mutex::new(HashMap::new()),
            routing_table: Mutex::new(routing_table),
        };

        if let Some(addr) = bootstrap_addr {
            println!(">Bootstrapping with node at address: {}", addr);
            node.initialize_routing_table(addr).await?;
        };
        node.routing_table.lock().await.print_table();
        Ok(node)
    }

    async fn generate_id() -> Bytes {
        let mut rng = rand::thread_rng();
        let mut id = vec![0u8; 20]; // 160 bits
        rng.fill_bytes(&mut id);
        Bytes::from(id)
    }

    
    async fn initialize_routing_table(&self, bootstrap_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Connect to the bootstrap node
        let endpoint = Endpoint::from_shared(format!("http://{}", bootstrap_addr))?;
        let channel = endpoint.connect().await?;
        let mut client = KademliaClient::new(channel);
    
        // Send a Ping request to get the bootstrap node's ID
        let ping_request = Request::new(PingRequest {node_address: bootstrap_addr.to_string()});
        println!("(REQUEST) --> Sending ping request to bootstrap node: {:?}", ping_request);
        let ping_response = client.ping(ping_request).await?;
        println!("(RESPONSE) --> Received ping response: {:?}", ping_response);
        let bootstrap_id = ping_response.into_inner().node_id; 
    
        // Now use the bootstrap node's ID to send a FindNode request
        let find_node_request = Request::new(FindNodeRequest {
            target_node_id: bootstrap_id, // Use the received ID to search
        });
        println!("(REQUEST) --> Sending find_node request to bootstrap node: {:?}", find_node_request);
        let response = client.find_node(find_node_request).await?;
        println!("(RESPONSE) --> Received find_node response: {:?}", response);
    
        // Update local routing table based on response
        self.update_routing_table(RoutingTable::from_proto_nodes(response.into_inner().nodes)).await;
    
        Ok(())
    }

    async fn update_routing_table(&self, nodes: Vec<NodeInfo>) {
        let mut routing_table = self.routing_table.lock().await;
        for node_info in nodes {
            routing_table.add_node(node_info, &self.id);
        }
    }
}

#[tonic::async_trait]
impl Kademlia for Node {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        println!("(REQUEST) --> Received ping request: {:?}", request);
        let response = PingResponse {
            is_online: true,
            node_id: self.id.clone().to_vec(),  // Convert Bytes to Vec<u8>
        };
        Ok(Response::new(response))
    }

    async fn store(&self, request: Request<StoreRequest>) -> Result<Response<StoreResponse>, Status> {
        println!("(REQUEST) --> Received store request: {:?}", request);
        let store_request = request.into_inner();
        let key = Bytes::from(store_request.key);
        let value = Bytes::from(store_request.value);
    
        let mut storage = self.storage.lock().await;
        // Insert the key-value pair into the local storage.
        storage.insert(key, value);
    
        // For now, we simply acknowledge the store operation succeeded without distributing it.
        Ok(Response::new(StoreResponse { success: true }))
    }

  async fn find_node(&self, request: Request<FindNodeRequest>) -> Result<Response<FindNodeResponse>, Status> {
        println!("(REQUEST) --> Received find_node request: {:?}", request);
        let target_id = Bytes::from(request.into_inner().target_node_id);

        // Debug: Log the target ID
        println!(">FindNode request for ID: {:?}", target_id);

        // Retrieve the closest nodes from the routing table
        let closest_nodes = {
            let routing_table = self.routing_table.lock().await;
            routing_table.find_closest(&target_id, &self.id)
        };

        // Debug: Log the found nodes
           if closest_nodes.is_empty() {
            println!(">No closest nodes found.");
        } else {
            for node in &closest_nodes {
                println!("  >Closest node found: ID = {:?}, Addr = {}", node.id, node.addr);
            }
        }
        

        // Convert NodeInfo to the appropriate response format, assuming a conversion method exists
        let proto_nodes = closest_nodes.iter().map(|node_info| {
            ProtoNodeInfo {
                id: node_info.id.clone().to_vec(), // Make sure this conversion is appropriate
                address: node_info.addr.to_string(), // Convert SocketAddr to string
            }
        }).collect::<Vec<_>>();

        // Prepare the response
        let response = FindNodeResponse {
            nodes: proto_nodes,
        };

        Ok(Response::new(response))
    }

    async fn find_value(&self, request: Request<FindValueRequest>) -> Result<Response<FindValueResponse>, Status> {
        println!("(REQUEST) --> Received find_value request: {:?}", request);
        // Implement FIND_VALUE logic here
        unimplemented!()
    }
}


pub async fn run_server(addr: &str, bootstrap_addr: Option<String>) -> Result<(), Box<dyn std::error::Error>> {

    let addr = addr.parse()?;
    let node = Node::new(addr,bootstrap_addr.as_deref()).await?;

    if let Some(bootstrap) = bootstrap_addr {
        // Connect to the bootstrap node and initialize routing table
        node.initialize_routing_table(&bootstrap).await?;
    }

    println!(">Server listening on {}", addr);
    //Start gRPC server
    Server::builder()
        .add_service(KademliaServer::new(node))
        .serve(addr)
        .await?;

    Ok(())
}
