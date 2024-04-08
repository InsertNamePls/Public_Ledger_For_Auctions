mod routing_table;

use std::collections::HashMap;
use tokio::sync::Mutex;
use bytes::Bytes;
use routing_table::RoutingTable;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use crate::kademlia::kademlia_server::{Kademlia, KademliaServer};
use crate::kademlia::{PingRequest, PingResponse, StoreRequest, StoreResponse, FindNodeRequest, FindNodeResponse, FindValueRequest, FindValueResponse};
use crate::kademlia::kademlia_client::KademliaClient;

pub struct Node {
    storage: Mutex<HashMap<Bytes, Bytes>>,
    routing_table: Mutex<RoutingTable>,
}


#[tonic::async_trait]
impl Kademlia for Node {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        println!("Got a request: {:?}", request);
        let response = PingResponse { is_online: true };
        Ok(Response::new(response))
    }

    async fn store(&self, request: Request<StoreRequest>) -> Result<Response<StoreResponse>, Status> {
        let store_request = request.into_inner();
        let key = Bytes::from(store_request.key);
        let value = Bytes::from(store_request.value);

        let mut storage = self.storage.lock().await;
        storage.insert(key.clone(), value);

        // Store the key-value pair in the closest nodes to the key in the network
        let closest_nodes = self.routing_table.lock().await.find_closest(&key);
        for node in closest_nodes {
            unimplemented!();
            //node.store(key.clone(), value.clone()).await?;
        }

        let response = StoreResponse { success: true };
        Ok(Response::new(response))
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


pub async fn run_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr.parse()?;
    let node = Node {
        storage: Mutex::new(HashMap::new()),
        routing_table: Mutex::new(RoutingTable::new()),
    };

    Server::builder()
        .add_service(KademliaServer::new(node))
        .serve(addr)
        .await?;

    Ok(())
}
