use crate::auction_app::auction::AuctionHouse;

use crate::auction_app::auction::Bid;
use crate::auction_app::auction::Notification;
use crate::auction_server::auction_handler::transaction_handler;
use crate::auction_tx::auction_tx_server::AuctionTx;
use crate::auction_tx::auction_tx_server::AuctionTxServer;
use crate::auction_tx::{
    GetAuctionsRequest, GetAuctionsResponse, SubmitTransactionRequest, SubmitTransactionResponse,
};
use crate::kademlia_node_search::node::Node;
use crate::kademlia_node_search::node_functions::routing_table;
use crate::kademlia_node_search::node_functions::routing_table::Bucket;
use std::env::split_paths;
use std::{fs, io};

use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{
    transport::{Identity, Server, ServerTlsConfig},
    Request, Response, Status,
};
type AuctionResult<T> = Result<Response<T>, Status>;

#[derive(Debug, Clone)]
pub struct AuctionsTxServer {
    shared_auction_house_state: Arc<Mutex<AuctionHouse>>,
    shared_kademlia_node: Arc<Mutex<Node>>,
}

#[tonic::async_trait]
impl AuctionTx for AuctionsTxServer {
    async fn submit_transaction(
        &self,
        request: Request<SubmitTransactionRequest>,
    ) -> AuctionResult<SubmitTransactionResponse> {
        let local_ip_address = request.remote_addr().clone();

        let message = request.into_inner().clone().transaction;
        let shared_node = self.shared_kademlia_node.clone();

        let routing_table = <Vec<Bucket> as Clone>::clone(
            &shared_node.lock().await.routing_table.lock().await.buckets,
        )
        .into_iter()
        .map(|x| {
            x.nodes
                .into_iter()
                .map(|node_info| node_info.addr.to_string())
                .collect::<Vec<String>>()
        })
        .flatten()
        .collect::<Vec<String>>();

        transaction_handler(
            message.clone(),
            &mut self.shared_auction_house_state.clone(),
            local_ip_address
                .clone()
                .unwrap()
                .to_string()
                .split(":")
                .next()
                .unwrap()
                .to_owned(),
            routing_table,
        )
        .await;
        Ok(Response::new(SubmitTransactionResponse { message }))
    }
    async fn get_auctions(
        &self,
        _: Request<GetAuctionsRequest>,
    ) -> AuctionResult<GetAuctionsResponse> {
        let auction_house_state = &self.shared_auction_house_state.lock().await;

        let mut auction_house_str: Vec<String> = Vec::new();
        for auction in auction_house_state.auctions.clone() {
            auction_house_str.push(serde_json::to_string(&auction).unwrap())
        }
        let auctions = serde_json::to_string(&auction_house_str).unwrap();

        Ok(Response::new(GetAuctionsResponse { auctions }))
    }
}
pub async fn auction_server(
    shared_auction_house: Arc<Mutex<AuctionHouse>>,
    kademlia_node: Arc<Mutex<Node>>,
) {
    let cert = std::fs::read_to_string("tls/server.crt");
    let key = std::fs::read_to_string("tls/server.key");

    let identity = Identity::from_pem(cert.unwrap(), key.unwrap());

    let addr = "0.0.0.0:3000".parse().unwrap();
    Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))
        .unwrap()
        .add_service(AuctionTxServer::new(AuctionsTxServer {
            shared_auction_house_state: shared_auction_house,
            shared_kademlia_node: kademlia_node,
        }))
        .serve(addr)
        .await
        .expect("error building server");
}
