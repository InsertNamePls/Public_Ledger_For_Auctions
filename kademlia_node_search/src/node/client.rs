use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tonic::transport::Endpoint;
use tonic::{Request, Status};
use crate::kademlia::kademlia_client::KademliaClient;
use crate::kademlia::{FindNodeRequest, FindNodeResponse, FindValueResponse, FindValueRequest, PingRequest, PingResponse, StoreRequest, StoreResponse};
use crate::node::crypto::Crypto;
use ring::signature::{Ed25519KeyPair, KeyPair};
use tokio::time::{timeout, Duration};
use crate::config::{TIMEOUT_MAX_ATTEMPTS, TIMEOUT_TIMER};
use colored::Colorize;
use rand::Rng;

#[derive(Clone)]
pub struct Client {
    nonce_map: Arc<Mutex<HashMap<Vec<u8>, i64>>>,
    crypto: Arc<Crypto>,
}

impl Client {
    pub fn new() -> Self {
        Client {
            nonce_map: Arc::new(Mutex::new(HashMap::new())),
            crypto: Arc::new(Crypto::new()),
        }
    }
    
    fn get_or_generate_nonce(&self, node_id: &Vec<u8>) -> i64 {
        let mut nonce_map = self.nonce_map.lock().unwrap();
        nonce_map.get(node_id).cloned().unwrap_or_else(|| {
            let nonce = rand::thread_rng().gen::<i64>();
            nonce_map.insert(node_id.clone(), nonce);
            nonce
        })
    }

    fn increment_nonce(&self, node_id: &Vec<u8>) {
        let mut nonce_map = self.nonce_map.lock().unwrap();
        if let Some(nonce) = nonce_map.get_mut(node_id) {
            *nonce += 1;
        }
    }

    async fn attempt_with_timeout<F, Fut, T>(mut attempt: F) -> Result<T, Status>
    where
        F: FnMut() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, Status>> + Send,
        T: Send,
    {
        for _ in 0..TIMEOUT_MAX_ATTEMPTS {
            let result = timeout(Duration::from_secs(TIMEOUT_TIMER), attempt()).await;
            match result {
                Ok(Ok(response)) => return Ok(response),
                Ok(Err(e)) => eprintln!("{}", format!("Attempt failed: {}", e).red()),
                Err(_) => eprintln!("{}", "Attempt timed out".red()),
            }
        }
        Err(Status::internal("All attempts to send the request failed"))
    }

    pub fn create_ping_request(&self, keypair: &Ed25519KeyPair, self_id: Vec<u8>, self_addr: String) -> PingRequest {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let nonce = self.get_or_generate_nonce(&self_id);
        let sender_public_key = keypair.public_key().as_ref().to_vec();
        // Message is always the concatenation of the node's timestamp and the public key and id
        let message = format!("{}{:?}{:?}",timestamp, sender_public_key, self_id).into_bytes();
        let signature = self.crypto.sign_message(keypair, &message);

        PingRequest {
            node_address: self_addr,
            timestamp,
            signature,
            sender_public_key,
            nonce,
            requester_node_id: self_id,
        }
    }

    pub async fn send_ping_request(&self, request: PingRequest, server_addr: String) -> Result<PingResponse, Status> {
        let endpoint = Endpoint::from_shared(format!("http://{}", server_addr))
            .map_err(|e| Status::internal(format!("Failed to create endpoint: {}", e)))?;

        let result = Client::attempt_with_timeout(|| {
            let endpoint = endpoint.clone();
            let request = request.clone();
            async move {
                let channel = endpoint.connect().await
                    .map_err(|e| Status::internal(format!("Failed to connect: {}", e)))?;
                let mut client = KademliaClient::new(channel);
                client.ping(request).await.map(|response| response.into_inner())
            }
        }).await;

        if result.is_ok() {
            self.increment_nonce(&request.requester_node_id);
        }

        result
    }

    pub fn create_find_node_request(&self, keypair: &Ed25519KeyPair, self_id: Vec<u8>, requester_node_address: String, target_node_id: Vec<u8>) -> FindNodeRequest {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let nonce = self.get_or_generate_nonce(&self_id);
        let sender_public_key = keypair.public_key().as_ref().to_vec();
        // Message is always the concatenation of the node's timestamp and the public key and id
        let message = format!("{}{:?}{:?}",timestamp, sender_public_key, self_id).into_bytes();
        let signature = self.crypto.sign_message(keypair, &message);

        FindNodeRequest {
            requester_node_id: self_id,
            requester_node_address: requester_node_address,
            target_node_id: target_node_id,
            timestamp,
            signature,
            sender_public_key,
            nonce,
        }
    }

    pub async fn send_find_node_request(&self, request: FindNodeRequest, target_address: String) -> Result<FindNodeResponse, Status> {
        let endpoint = Endpoint::from_shared(format!("http://{}", target_address))
            .map_err(|e| Status::internal(format!("Failed to create endpoint: {}", e)))?;

        let result = Client::attempt_with_timeout(|| {
            let endpoint = endpoint.clone();
            let request = request.clone();
            async move {
                let channel = endpoint.connect().await
                    .map_err(|e| Status::internal(format!("Failed to connect: {}", e)))?;
                let mut client = KademliaClient::new(channel);
                client.find_node(request).await.map(|response| response.into_inner())
            }
        }).await;

        if result.is_ok() {
            self.increment_nonce(&request.requester_node_id);
        }

        result
    }

    pub fn create_store_node_request(&self, keypair: &Ed25519KeyPair, self_id: Vec<u8>, key: Vec<u8>, value: Vec<u8>) -> StoreRequest {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let nonce = self.get_or_generate_nonce(&self_id);
        let sender_public_key = keypair.public_key().as_ref().to_vec();
        // Message is always the concatenation of the node's timestamp and the public key and id
        let message = format!("{}{:?}{:?}",timestamp, sender_public_key, self_id).into_bytes();
        let signature = self.crypto.sign_message(keypair, &message);

        StoreRequest {
            key,
            value,
            timestamp,
            signature,
            sender_public_key,
            nonce,
            requester_node_id: self_id,
        }
    }

    pub async fn send_store_request(&self, request: StoreRequest, target_address: String) -> Result<StoreResponse, Status> {
        let endpoint = Endpoint::from_shared(format!("http://{}", target_address))
            .map_err(|e| Status::internal(format!("Failed to create endpoint: {}", e)))?;

        let result = Client::attempt_with_timeout(|| {
            let endpoint = endpoint.clone();
            let request = request.clone();
            async move {
                let channel = endpoint.connect().await
                    .map_err(|e| Status::internal(format!("Failed to connect: {}", e)))?;
                let mut client = KademliaClient::new(channel);
                client.store(request).await.map(|response| response.into_inner())
            }
        }).await;

        if result.is_ok() {
            self.increment_nonce(&request.requester_node_id);
        }

        result
    }

    pub fn create_find_value_request(&self, keypair: &Ed25519KeyPair, self_id: Vec<u8>, key: Vec<u8>) -> FindValueRequest {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let nonce = self.get_or_generate_nonce(&self_id);
        let sender_public_key = keypair.public_key().as_ref().to_vec();
        // Message is always the concatenation of the node's timestamp and the public key and id
        let message = format!("{}{:?}{:?}",timestamp, sender_public_key, self_id).into_bytes();
        let signature = self.crypto.sign_message(keypair, &message);

        FindValueRequest {
            key,
            timestamp,
            signature,
            sender_public_key,
            nonce,
            requester_node_id: self_id,
        }
    }

    pub async fn send_find_value_request(&self, request: FindValueRequest, target_address: String) -> Result<FindValueResponse, Status> {
        let endpoint = Endpoint::from_shared(format!("http://{}", target_address))
            .map_err(|e| Status::internal(format!("Failed to create endpoint: {}", e)))?;

        let result = Client::attempt_with_timeout(|| {
            let endpoint = endpoint.clone();
            let request = request.clone();
            async move {
                let channel = endpoint.connect().await
                    .map_err(|e| Status::internal(format!("Failed to connect: {}", e)))?;
                let mut client = KademliaClient::new(channel);
                client.find_value(request).await.map(|response| response.into_inner())
            }
        }).await;

        if result.is_ok() {
            self.increment_nonce(&request.requester_node_id);
        }

        result
    }
}
