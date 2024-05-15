use std::time::{SystemTime, UNIX_EPOCH};
use tonic::transport::{Endpoint};
use tonic::{Request, Status};
use crate::kademlia::kademlia_client::KademliaClient;
use crate::kademlia::{PingRequest, PingResponse, FindNodeRequest, FindNodeResponse};
use crate::node::crypto::Crypto;
use ring::signature::{Ed25519KeyPair, KeyPair};
use bytes::Bytes;

pub struct Client;

impl Client {
    pub fn create_ping_request(keypair: &Ed25519KeyPair, addr: String) -> PingRequest {
        let node_address = addr;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let message = format!("{}{}", node_address, timestamp).into_bytes();
        let signature = Crypto::sign_message(keypair, &message);
        let sender_public_key = keypair.public_key().as_ref().to_vec();

        PingRequest {
            node_address,
            timestamp,
            signature,
            sender_public_key,
        }
    }

    pub async fn send_ping_request(keypair: &Ed25519KeyPair, addr: String, server_addr: String) -> Result<PingResponse, Status> {
        let endpoint = Endpoint::from_shared(format!("http://{}", server_addr))
            .map_err(|e| Status::internal(format!("Failed to create endpoint: {}", e)))?;
        let channel = endpoint.connect().await
            .map_err(|e| Status::internal(format!("Failed to connect: {}", e)))?;
        let mut client = KademliaClient::new(channel);

        let ping_request = Self::create_ping_request(keypair, addr);
        let request = Request::new(ping_request);
        let response = client.ping(request).await?.into_inner();

        Ok(response)
    }

    pub async fn send_find_node_request(target_node_id: Vec<u8>, target_address: String,requester_node_id: Vec<u8>, requester_node_address: String) -> Result<FindNodeResponse, Status> {
        let endpoint = Endpoint::from_shared(format!("http://{}", target_address))
            .map_err(|e| Status::internal(format!("Failed to create endpoint: {}", e)))?;
        let channel = endpoint.connect().await
            .map_err(|e| Status::internal(format!("Failed to connect: {}", e)))?;
        let mut client = KademliaClient::new(channel);

        let find_node_request = FindNodeRequest {
            requester_node_id: requester_node_id.to_vec(),
            requester_node_address: requester_node_address.clone(),
            target_node_id: target_node_id.to_vec(),
        };
        let request = Request::new(find_node_request);
        let response = client.find_node(request).await?.into_inner();

        Ok(response)
    }

    
}
