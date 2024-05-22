pub mod auction_app;
pub mod auction_server;
pub mod cryptography;
pub mod kademlia_node_search;
pub mod kademlia {
    tonic::include_proto!("kademlia");
}
pub mod blockchain_grpc {
    tonic::include_proto!("blockchain_grpc");
}
pub mod auction_tx {
    tonic::include_proto!("auction_tx");
}

pub mod notification_tx {
    tonic::include_proto!("notification_tx");
}
