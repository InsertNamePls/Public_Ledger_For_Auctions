use bytes::Bytes;
use std::collections::VecDeque;
use std::net::SocketAddr;
use crate::kademlia::NodeInfo as ProtoNodeInfo;


const K: usize = 20; // The maximum number of nodes in a bucket
const N_BITS: usize = 160; // The number of bits in the node ID

#[derive(Clone)]
pub struct NodeInfo {
    id: Bytes,
    addr: SocketAddr,
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
        let table_id = &self.buckets[0].nodes[0].id; // Assuming the first bucket and first node as the owner
        let xor_distance = Self::xor_distance(id, table_id);
        let leading_zeros = xor_distance.leading_zeros() as usize;
        leading_zeros
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
    
}