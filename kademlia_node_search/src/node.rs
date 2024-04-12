mod routing_table;

use std::collections::HashMap;
use tokio::sync::Mutex;
use bytes::Bytes;
use routing_table::RoutingTable;
use tonic::transport::{Endpoint, Server};
use tonic::{Request, Response, Status};
use crate::kademlia::kademlia_client::KademliaClient;
use crate::kademlia::kademlia_server::{Kademlia, KademliaServer};
use crate::kademlia::{PingRequest, PingResponse, StoreRequest, StoreResponse, FindNodeRequest, FindNodeResponse, FindValueRequest, FindValueResponse};
use rand::RngCore;

use self::routing_table::NodeInfo;

pub struct Node {
    id: Bytes,
    storage: Mutex<HashMap<Bytes, Bytes>>,
    routing_table: Mutex<RoutingTable>,
}

impl Node {
    pub async fn new(bootstrap_addr: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let node = Node {
            id: Self::generate_id().await,
            storage: Mutex::new(HashMap::new()),
            routing_table: Mutex::new(RoutingTable::new()),
        };

        if let Some(addr) = bootstrap_addr {
            node.initialize_routing_table(addr).await?;
        }

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
        let ping_response = client.ping(ping_request).await?;
        let bootstrap_id = ping_response.into_inner().node_id; 
    
        // Now use the bootstrap node's ID to send a FindNode request
        let find_node_request = Request::new(FindNodeRequest {
            target_node_id: bootstrap_id, // Use the received ID to search
        });

        let response = client.find_node(find_node_request).await?;
    
        // Update local routing table based on response
        self.update_routing_table(RoutingTable::from_proto_nodes(response.into_inner().nodes)).await;
    
        Ok(())
    }

    async fn update_routing_table(&self, nodes: Vec<NodeInfo>) {
        let mut routing_table = self.routing_table.lock().await;
        for node_info in nodes {
            routing_table.add_node(node_info);
        }
    }
}

#[tonic::async_trait]
impl Kademlia for Node {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        println!("Received Ping request: {:?}", request);
        let response = PingResponse {
            is_online: true,
            node_id: self.id.clone().to_vec(),  // Convert Bytes to Vec<u8>
        };
        Ok(Response::new(response))
    }

    async fn store(&self, request: Request<StoreRequest>) -> Result<Response<StoreResponse>, Status> {
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
        // Implement FIND_NODE logic here
        unimplemented!()
    }

    async fn find_value(&self, request: Request<FindValueRequest>) -> Result<Response<FindValueResponse>, Status> {
        // Implement FIND_VALUE logic here
        unimplemented!()
    }
}


pub async fn run_server(addr: &str, bootstrap_addr: Option<String>) -> Result<(), Box<dyn std::error::Error>> {

    let addr = addr.parse()?;
    let node = Node::new(bootstrap_addr.as_deref()).await?;

    if let Some(bootstrap) = bootstrap_addr {
        // Connect to the bootstrap node and initialize routing table
        node.initialize_routing_table(&bootstrap).await?;
    }

    println!("Generated Node ID: {:?}", node.id);

    //Start gRPC server
    Server::builder()
        .add_service(KademliaServer::new(node))
        .serve(addr)
        .await?;

    Ok(())
}
