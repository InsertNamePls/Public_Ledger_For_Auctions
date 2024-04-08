use bytes::Bytes;
use std::collections::VecDeque;
use std::net::SocketAddr;

const K: usize = 20; // The maximum number of nodes in a bucket

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
            buckets: vec![Bucket::new(); 160], // 160 is the number of bits in the node ID
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
        // This is a placeholder implementation. In a real implementation, you would calculate the
        // bucket index based on the XOR distance between the id and the ID of the node that owns
        // this routing table.
        0
    }
}