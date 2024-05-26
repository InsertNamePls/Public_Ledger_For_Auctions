use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use crate::config::K;
use crate::node::routing_table::RoutingTable;
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
        let requester_address = ping_request.node_address; // Assuming node_address is a field in PingRequest
    
        // Message is always the concatenation of the node's timestamp and the public key and id
        let message = format!("{}{:?}{:?}", ping_request.timestamp, ping_request.sender_public_key, ping_request.requester_node_id).into_bytes();
        let sender_public_key = ping_request.sender_public_key.as_ref();
    
        // Ensure the request is valid
        if node.lock().await.crypto.validate_request(timestamp, nonce, &requester_id, &message, &ping_request.signature, sender_public_key) {
            let node_lock = node.lock().await;
    
            // Add the requester's information to the routing table if it's not already there
            let mut routing_table = node_lock.routing_table.lock().await;
            if !routing_table.contains(&requester_id) {
                let requester_node_info = NodeInfo::new(
                    requester_id.clone(),
                    requester_address.parse::<SocketAddr>().unwrap()
                );
                routing_table.add_node(requester_node_info, &node_lock.id);
                println!("{}", "Added requester's information to routing table.".green());
                println!("{}", "Updated routing table:".green());
                routing_table.print_table();
            }
    
            let response = PingResponse {
                is_online: true,
                node_id: node_lock.id.clone().to_vec(),
            };
            routing_table.adjust_reputation(&requester_id, 1);
            Ok(Response::new(response))
        } else {
            let node_lock = node.lock().await;
            node_lock.routing_table.lock().await.adjust_reputation(&requester_id, -1);
            Err(Status::unauthenticated("Invalid request"))
        }
    }

    pub async fn handle_find_node(node: Arc<Mutex<Node>>, request: Request<FindNodeRequest>) -> Result<Response<FindNodeResponse>, Status> {
        let find_node_request = request.into_inner();
        let target_id = Bytes::from(find_node_request.target_node_id.clone());
        let requester_id = Bytes::from(find_node_request.requester_node_id.clone());
        let requester_address = find_node_request.requester_node_address.clone();
        let nonce = find_node_request.nonce;
        let timestamp = find_node_request.timestamp;
    
        // Message is always the concatenation of the node's timestamp and the public key and id
        let message = format!("{}{:?}{:?}", find_node_request.timestamp, find_node_request.sender_public_key, find_node_request.requester_node_id).into_bytes();
    
        // Ensure the request is valid
        if !node.lock().await.crypto.validate_request(timestamp, nonce, &requester_id, &message, &find_node_request.signature, &find_node_request.sender_public_key) {
            let node = node.lock().await;
            node.routing_table.lock().await.adjust_reputation(&requester_id, -1);
            return Err(Status::unauthenticated("Invalid request"));
        }
    
        let mut node_lock = node.lock().await;
        node_lock.routing_table.lock().await.adjust_reputation(&requester_id, 1);
        println!("{}", format!("Received find_node request from [{}]: target_id {}, requester_id {}",
            requester_address,
            target_id.encode_hex::<String>(),
            requester_id.encode_hex::<String>()).blue()
        );
    
        // Add the requester's information to the routing table if it's not already there
        let mut routing_table = node_lock.routing_table.lock().await;
        if !routing_table.contains(&requester_id) {
            let requester_node_info = NodeInfo::new(
                requester_id.clone(),
                requester_address.parse::<SocketAddr>().unwrap()
            );
            routing_table.add_node(requester_node_info, &node_lock.id);
            println!("{}", "Added requester's information to routing table.".green());
            println!("{}", "Updated routing table:".green());
            routing_table.print_table();
        }
    
        // Check if the target node is in the local routing table or the node itself
        let target_node_info = routing_table.get_node(&target_id);
        if target_id == node_lock.id || target_node_info.is_some() {
            println!("Target node is the current node or found locally: {}", node_lock.addr);
    
            // Return all nodes in the local routing table
            let all_nodes = routing_table.get_all_nodes();
            let proto_nodes = all_nodes.iter().map(|node_info| ProtoNodeInfo {
                id: node_info.id.clone().to_vec(),
                address: node_info.addr.to_string(),
            }).collect::<Vec<_>>();
    
            return Ok(Response::new(FindNodeResponse { nodes: proto_nodes }));
        }
    
        println!("Target node not found locally. Performing iterative lookup.");
    
        // Iterative lookup
        let closest_nodes = routing_table.find_closest(&target_id);
        let mut queried_nodes = vec![requester_id.clone()];
        let mut all_discovered_nodes = routing_table.get_all_nodes();
    
        drop(routing_table); // Release the lock on the routing table
        drop(node_lock); // Release the lock on the node
    
        let mut closest_seen = closest_nodes.clone();
        let mut new_nodes_found = true;
    
        while new_nodes_found {
            new_nodes_found = false;
            for node_info in closest_nodes.iter() {
                if queried_nodes.contains(&node_info.id) {
                    continue;
                }
    
                queried_nodes.push(node_info.id.clone());
    
                // Check if the node is not the same as the current node before forwarding the request
                if node_info.id == node.lock().await.id {
                    continue;
                }
    
                // Send FindNode request to each of the closest nodes
                let client_addr = node_info.addr.to_string();
                let find_node_request = {
                    let node_lock = node.lock().await;
                    node_lock.client.create_find_node_request(
                        &node_lock.keypair,
                        node_lock.id.to_vec(),
                        node_lock.addr.to_string(),
                        target_id.to_vec(),
                    )
                };
    
                match node.lock().await.client.send_find_node_request(find_node_request, client_addr.clone()).await {
                    Ok(response) => {
                        let nodes_from_response = RoutingTable::from_proto_nodes(response.nodes);
                        for new_node in nodes_from_response {
                            if !closest_seen.contains(&new_node) && new_node.id != node.lock().await.id {
                                closest_seen.push(new_node.clone());
                                all_discovered_nodes.push(new_node.clone());
                                new_nodes_found = true;
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to send find_node request to {}: {}", client_addr, e);
                        node.lock().await.routing_table.lock().await.remove_node(&node_info.id);
                    }
                }
            }
    
            // Sort and truncate to K closest nodes
            closest_seen.sort_by_key(|node| RoutingTable::xor_distance(&node.id, &target_id));
            closest_seen.truncate(K);
        }
    
        // If all_discovered_nodes is still empty, add the current node
        if all_discovered_nodes.is_empty() {
            let node_lock = node.lock().await;
            all_discovered_nodes.push(NodeInfo::new(node_lock.id.clone(), node_lock.addr));
        }
    
        let proto_nodes = all_discovered_nodes.iter().map(|node_info| ProtoNodeInfo {
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
}
