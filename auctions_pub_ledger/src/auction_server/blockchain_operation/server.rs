use crate::auction_server::blockchain::Blockchain;

use std::sync::Arc;
use std::vec::Vec;
use tokio::sync::Mutex;

use crate::auction_server::blockchain_operation::client::BlockchainServer;
use crate::blockchain_grpc::blockchain_grpc_server::BlockchainGrpcServer;

use tonic::transport::{Identity, Server, ServerTlsConfig};
// blockchain Server
pub async fn blockchain_server(share_blockchain_vector: Arc<Mutex<Vec<Blockchain>>>) {
    let cert = std::fs::read_to_string("tls/server.crt");
    let key = std::fs::read_to_string("tls/server.key");

    let identity = Identity::from_pem(cert.unwrap(), key.unwrap());

    let addr = "0.0.0.0:3001".parse().unwrap();

    Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))
        .unwrap()
        .add_service(BlockchainGrpcServer::new(BlockchainServer {
            shared_blockchain_state: share_blockchain_vector,
        }))
        .serve(addr)
        .await
        .unwrap();
}
