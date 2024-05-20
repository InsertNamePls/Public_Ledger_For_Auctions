use bytes::Bytes;
use rand::seq::IteratorRandom;
use rand::thread_rng;
use std::collections::VecDeque;
use std::net::SocketAddr;
use crate::kademlia::NodeInfo as ProtoNodeInfo;
use crate::config::{N_BITS,K};


#[derive(Clone,Debug)]
pub struct NodeInfo {
    pub(crate) id: Bytes,
    pub(crate) addr: SocketAddr,
}

#[derive(Clone,Debug)]
pub struct Bucket {
    nodes: VecDeque<NodeInfo>,
}

impl Bucket {
    pub fn new() -> Self {
        Self {
            nodes: VecDeque::with_capacity(K),
        }
    }

    pub fn add(&mut self, node: NodeInfo) {
        if self.nodes.len() == K {
            self.nodes.pop_front();
        }
        self.nodes.push_back(node);
    }

    pub fn remove(&mut self, node_id: &Bytes) {
        self.nodes.retain(|node| &node.id != node_id);
    }
}

#[derive(Debug)]
pub struct RoutingTable {
    buckets: Vec<Bucket>,
    own_id: Bytes,  // Store the node's own ID for reference
}

impl RoutingTable {
    pub fn new(own_id: Bytes) -> Self {
        Self {
            buckets: vec![Bucket::new(); N_BITS],
            own_id,
        }
    }


    // Add the node's own ID as a parameter for all necessary functions
    fn calculate_bucket_index(&self, id: &Bytes, own_id: &Bytes) -> usize {
        let xor_distance = Self::xor_distance(id, own_id);
        xor_distance.leading_zeros() as usize % N_BITS
    }

    fn xor_distance(id1: &Bytes, id2: &Bytes) -> u128 {
        let mut result = 0;
        for (byte1, byte2) in id1.iter().zip(id2.iter()) {
            result = (result << 8) | (*byte1 ^ *byte2) as u128;
        }
        result
    }

    pub fn add_node(&mut self, node: NodeInfo, own_id: &Bytes) {
        if node.id == self.own_id {
            return;
        }
        let bucket_index = self.calculate_bucket_index(&node.id, own_id);
        self.buckets[bucket_index].add(node);
    }

    pub fn remove_node(&mut self, node_id: &Bytes) {
        let bucket_index = self.calculate_bucket_index(node_id, &self.own_id);
        self.buckets[bucket_index].remove(node_id);
    }

    pub fn find_closest(&self, target_id: &Bytes) -> Vec<NodeInfo> {
        let primary_index = self.calculate_bucket_index(target_id, &self.own_id);
        let mut closest_nodes = Vec::new();

        let mut distance = 0;
        while closest_nodes.len() < K && (primary_index >= distance || primary_index + distance < N_BITS) {
            // Check lower bucket if it exists and isn't the same as the higher one
            if primary_index >= distance {
                closest_nodes.extend(self.buckets[primary_index - distance].nodes.iter().cloned());
            }
            // Check higher bucket if it exists
            if primary_index + distance < N_BITS && distance > 0 {
                closest_nodes.extend(self.buckets[primary_index + distance].nodes.iter().cloned());
            }
            // Increment distance for the next iteration to check the next set of buckets
            distance += 1;
        }

        // Sort and truncate to return the closest K nodes
        closest_nodes.sort_by_key(|node| Self::xor_distance(&node.id, target_id));
        closest_nodes.truncate(K);
        closest_nodes
    }

    pub fn from_proto_nodes(proto_nodes: Vec<ProtoNodeInfo>) -> Vec<NodeInfo> {
        let mut nodes: Vec<NodeInfo> = Vec::new();
        for proto_node in proto_nodes {
            let id = Bytes::from(proto_node.id);
            let addr = proto_node.address.parse().expect("Failed to parse SocketAddr");
            let node_info = NodeInfo { id, addr };
            nodes.push(node_info);
        }
        nodes
    }

    pub fn random_node(&self) -> Option<&NodeInfo> {
        self.buckets.iter()
            .flat_map(|bucket| &bucket.nodes) // Iterate over all nodes in all buckets
            .filter(|node_info| node_info.id != self.own_id) // Exclude the node's own ID
            .choose(&mut thread_rng())  // Randomly select one node
    }

    // Function to check if a node is already in the routing table
    pub fn contains(&self, node_id: &Bytes) -> bool {
        self.buckets.iter().any(|bucket| {
            bucket.nodes.iter().any(|node_info| &node_info.id == node_id)
        })
    }


    pub fn print_table(&self) {
        let routing_table = &self.buckets;
        println!("Routing Table:");
        println!("{:<10} | {:<64} | {:<30}", "Bucket", "Node ID", "Address");
        println!("{:-<110}", "");  // Print a dividing line
    
        for (i, bucket) in routing_table.iter().enumerate() {
            if !bucket.nodes.is_empty() {
                for node in &bucket.nodes {
                    let node_id_hex = format!("{:x}", node.id);  // Convert Bytes to a hex string for better readability
                    println!("{:<10} | {:<64} | {:<30}", i, node_id_hex, node.addr);
                }
            }
        }
    }
    
    
}