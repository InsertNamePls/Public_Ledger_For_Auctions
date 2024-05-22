use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use crate::node::Node;
use crate::kademlia::{NodeInfo as ProtoNodeInfo, PingRequest, PingResponse, StoreRequest, StoreResponse, FindNodeRequest, FindNodeResponse, FindValueRequest, FindValueResponse};
use bytes::Bytes;
use hex::ToHex;
use colored::*;
use super::routing_table::NodeInfo;

pub struct RequestHandler;

impl RequestHandler {
    pub async fn handle_ping(node: Arc<Mutex<Node>>, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let ping_request = request.into_inner();
        let timestamp = ping_request.timestamp;
        let nonce = ping_request.nonce;
        let requester_id = Bytes::from(ping_request.requester_node_id.clone());

        // Message is always the concatenation of the node's timestamp and the public key and id
        let message = format!("{}{:?}{:?}",ping_request.timestamp, ping_request.sender_public_key, ping_request.requester_node_id).into_bytes();
        let sender_public_key = ping_request.sender_public_key.as_ref();

        // Ensure the request is valid
        if node.lock().await.crypto.validate_request(timestamp, nonce, &requester_id, &message, &ping_request.signature, sender_public_key) {
            let node = node.lock().await;
            let response = PingResponse {
                is_online: true,
                node_id: node.id.clone().to_vec(),
            };
            node.routing_table.lock().await.adjust_reputation(&requester_id, 1);
            Ok(Response::new(response))
        } else {
            let node = node.lock().await;
            node.routing_table.lock().await.adjust_reputation(&requester_id, -1);
            Err(Status::unauthenticated("Invalid request"))
        }
    }

    pub async fn handle_store(node: Arc<Mutex<Node>>, request: Request<StoreRequest>) -> Result<Response<StoreResponse>, Status> {
        let store_request = request.into_inner();
        let key = Bytes::from(store_request.key.clone());
        let value = Bytes::from(store_request.value.clone());
        let nonce = store_request.nonce;
        let requester_id = Bytes::from(store_request.requester_node_id.clone());
        let timestamp = store_request.timestamp;
    
        // Message is always the concatenation of the node's timestamp and the public key and id
        let message = format!("{}{:?}{:?}", store_request.timestamp, store_request.sender_public_key, store_request.requester_node_id).into_bytes();
    
        // Ensure the request is valid
        if !node.lock().await.crypto.validate_request(timestamp, nonce, &requester_id, &message, &store_request.signature, &store_request.sender_public_key) {
            let node = node.lock().await;
            node.routing_table.lock().await.adjust_reputation(&requester_id, -1);            
            return Err(Status::unauthenticated("Invalid request"));
        }

        let node = node.lock().await;
        let mut storage = node.storage.lock().await;
        let should_forward = match storage.get(&key) {
            Some(existing_value) if existing_value == &value => false,
            _ => {
                storage.insert(key.clone(), value.clone());
                let key_hex = format!("{:x}", key);
                let value_hex = format!("{:x}", value);
                println!("Added <key:value> to local storage: <{}:{}>", key_hex, value_hex);
                node.routing_table.lock().await.adjust_reputation(&requester_id, 1);            
                true
            }
        };
    
        if should_forward {
            let closest_nodes = {
                let routing_table = node.routing_table.lock().await;
                routing_table.find_closest(&key)
            };
    
            let client = node.client.clone(); // clone the client for use within the async block
    
            for node_info in closest_nodes.iter() {
                let client_addr = node_info.addr.to_string();
    
                let forward_store_request = client.create_store_node_request(
                    &node.keypair, // use the current node's keypair
                    node.id.to_vec(), // use the current node's id
                    key.to_vec(),
                    value.to_vec()
                );

                if client_addr != node.addr.to_string() {
                    let client_clone = node.client.clone();
                    tokio::spawn(async move {
                        let result = client_clone.send_store_request(forward_store_request, client_addr.clone()).await;
                        if let Err(e) = result {
                            eprintln!("Failed to forward store request to {}: {}", client_addr, e);
                        } else {
                            println!("Successfully forwarded store request to {}", client_addr);
                        }
                    });
                }
            }
        }
    
        Ok(Response::new(StoreResponse { success: true }))
    }
    
    

    pub async fn handle_find_node(node: Arc<Mutex<Node>>, request: Request<FindNodeRequest>) -> Result<Response<FindNodeResponse>, Status> {
        let find_node_request = request.into_inner();
        let target_id = Bytes::from(find_node_request.target_node_id.clone());
        let requester_id = Bytes::from(find_node_request.requester_node_id.clone());
        let requester_address = find_node_request.requester_node_address.clone();
        let nonce = find_node_request.nonce;
        let timestamp = find_node_request.timestamp;

        // Message is always the concatenation of the node's timestamp and the public key and id
        let message = format!("{}{:?}{:?}",find_node_request.timestamp, find_node_request.sender_public_key, find_node_request.requester_node_id).into_bytes();

        // Ensure the request is valid
        if !node.lock().await.crypto.validate_request(timestamp, nonce, &requester_id, &message, &find_node_request.signature, &find_node_request.sender_public_key) {
            let node = node.lock().await;
            node.routing_table.lock().await.adjust_reputation(&requester_id, -1);
            return Err(Status::unauthenticated("Invalid request"));
        }

        let node = node.lock().await;
        node.routing_table.lock().await.adjust_reputation(&requester_id, 1);
        println!("{}", format!("Received find_node request from [{}]: target_id {}, requester_id {}", 
        requester_address, 
        target_id.encode_hex::<String>(), 
        requester_id.encode_hex::<String>()).blue());

        let closest_nodes = {
            let routing_table = node.routing_table.lock().await;
            routing_table.find_closest(&target_id)
        };

        let mut routing_table = node.routing_table.lock().await;
        if !routing_table.contains(&requester_id) {
            let requester_node_info = NodeInfo::new(
                requester_id.clone(),
                requester_address.parse::<SocketAddr>().unwrap()
            );
            routing_table.add_node(requester_node_info, &node.id);
            println!("{}", "Added requester's information to routing table.".green());
            println!("{}", "Updated routing table:".green());
            routing_table.print_table();
        }

        let proto_nodes = closest_nodes.iter().map(|node_info| ProtoNodeInfo {
            id: node_info.id.clone().to_vec(),
            address: node_info.addr.to_string(),
        }).collect::<Vec<_>>();

        Ok(Response::new(FindNodeResponse { nodes: proto_nodes }))
    }

    pub async fn handle_find_value(node: Arc<Mutex<Node>>, request: Request<FindValueRequest>) -> Result<Response<FindValueResponse>, Status> {
        let find_value_request = request.into_inner();
        let key = Bytes::from(find_value_request.key);
        let nonce = find_value_request.nonce;
        let requester_id = Bytes::from(find_value_request.requester_node_id.clone());
        let timestamp = find_value_request.timestamp;

        // Message is always the concatenation of the node's timestamp and the public key and id
        let message = format!("{}{:?}{:?}",find_value_request.timestamp, find_value_request.sender_public_key, find_value_request.requester_node_id).into_bytes();

        // Ensure the request is valid
        if !node.lock().await.crypto.validate_request(timestamp, nonce, &requester_id, &message, &find_value_request.signature, &find_value_request.sender_public_key) {
            let node = node.lock().await;
            node.routing_table.lock().await.adjust_reputation(&requester_id, -1);
            return Err(Status::unauthenticated("Invalid request"));
        }

        let node = node.lock().await;
        node.routing_table.lock().await.adjust_reputation(&requester_id, 1);
        println!("{}", format!("Key requested: {:?}", key).yellow());

        let storage = node.storage.lock().await;
        if let Some(value) = storage.get(&key) {
            return Ok(Response::new(FindValueResponse {
                value: value.clone().to_vec(),
                nodes: vec![],
            }));
        }

        let routing_table = node.routing_table.lock().await;
        let closest_nodes = routing_table.find_closest(&key);

        let proto_nodes = closest_nodes.iter().map(|node_info| {
            ProtoNodeInfo {
                id: node_info.id.clone().to_vec(),
                address: node_info.addr.to_string(),
            }
        }).collect::<Vec<_>>();

        Ok(Response::new(FindValueResponse {
            value: Vec::new(),
            nodes: proto_nodes,
        }))
    }
}
