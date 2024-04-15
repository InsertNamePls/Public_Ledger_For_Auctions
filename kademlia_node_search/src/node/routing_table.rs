use bytes::Bytes;
use std::collections::VecDeque;
use std::net::SocketAddr;
use crate::kademlia::NodeInfo as ProtoNodeInfo;


const K: usize = 20; // The maximum number of nodes in a bucket
const N_BITS: usize = 160; // The number of bits in the node ID

#[derive(Clone)]
pub struct NodeInfo {
    pub(crate) id: Bytes,
    pub(crate) addr: SocketAddr,
}

#[derive(Clone)]
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
}

pub struct RoutingTable {
    buckets: Vec<Bucket>,
}

impl RoutingTable {
    pub fn new() -> Self {
        Self {
            buckets: vec![Bucket::new(); N_BITS],
        }
    }

    pub fn add_node(&mut self, node: NodeInfo) {
        let bucket_index = self.calculate_bucket_index(&node.id);
        self.buckets[bucket_index].add(node);
    }

    pub fn find_closest(&self, id: &Bytes) -> Vec<NodeInfo> {
        let bucket_index = self.calculate_bucket_index(id);
        self.buckets[bucket_index].nodes.iter().cloned().collect()
    }

    fn calculate_bucket_index(&self, id: &Bytes) -> usize {
        if let Some(first_bucket) = self.buckets.first() {
            if let Some(first_node) = first_bucket.nodes.front() {
                let table_id = &first_node.id;
                let xor_distance = Self::xor_distance(id, table_id);
                xor_distance.leading_zeros() as usize
            } else {
                // Return a default or calculated bucket index if there are no nodes yet
                0 // or some other logic
            }
        } else {
            // No buckets scenario, which should not happen if buckets are initialized correctly
            0 // or some other logic
        }
    }

    fn xor_distance(id1: &Bytes, id2: &Bytes) -> u128 {
        let mut result = 0;
        for (byte1, byte2) in id1.iter().zip(id2.iter()) {
            result = (result << 8) | (byte1 ^ byte2) as u128;
        }
        result
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

    pub fn print_table(&self) {
        let routing_table = &self.buckets;
        println!("Routing Table:");
        for (i, bucket) in routing_table.iter().enumerate() {
            if !bucket.nodes.is_empty() {
                println!("Bucket {}: ", i);
                for node in &bucket.nodes {
                    println!("\tNode ID: {:?}, Address: {}", node.id, node.addr);
                }
            }
        }
    }

    
}