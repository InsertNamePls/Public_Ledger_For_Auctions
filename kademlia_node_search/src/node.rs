// node.rs

use tonic::{Request, Response, Status};
use crate::kademlia::{kademlia_server::Kademlia, PingRequest, PingResponse, StoreRequest, StoreResponse, FindNodeRequest, FindNodeResponse, FindValueRequest, FindValueResponse};

pub struct Node {}

#[tonic::async_trait]
impl Kademlia for Node {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        println!("Got a request: {:?}", request);
        let response = PingResponse { is_online: true };
        Ok(Response::new(response))
    }

    async fn store(&self, request: Request<StoreRequest>) -> Result<Response<StoreResponse>, Status> {
        // Implement storing logic here
        unimplemented!()
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
