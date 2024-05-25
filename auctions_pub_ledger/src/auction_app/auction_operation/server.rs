use crate::auction_app::auction::AuctionHouse;
use crate::auction_app::auction::Bid;
use crate::auction_app::auction_operation::client::TransactionInfo;
use crate::auction_app::user::{load_users_from_file, save_user_in_file, User};
use crate::auction_server::auction_handler::transaction_handler;
use crate::auction_tx::auction_tx_server::AuctionTx;
use crate::auction_tx::auction_tx_server::AuctionTxServer;
use crate::auction_tx::{
    CreateUsersRequest, CreateUsersResponse, GetAuctionsRequest, GetAuctionsResponse,
    GetUsersRequest, GetUsersResponse, SubmitTransactionRequest, SubmitTransactionResponse,
    UpdateUsersRequest, UpdateUsersResponse,
};
use crate::kademlia_node_search::node::Node;
use crate::kademlia_node_search::node_functions::routing_table::Bucket;
use std::fs::{self};
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

        let transaction_info: TransactionInfo =
            serde_json::from_str(&message).expect("Error getting transaction info");

        transaction_handler(
            transaction_info.transaction,
            &mut self.shared_auction_house_state.clone(),
            transaction_info.subscriber_addrs,
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
    async fn create_users(
        &self,
        request: Request<CreateUsersRequest>,
    ) -> AuctionResult<CreateUsersResponse> {
        let user_str_request = request.into_inner().clone().user;

        let user: User = serde_json::from_str(&user_str_request).unwrap();
        let file_path = format!("users/{}.json", user.uid);

        save_user_in_file(&user_str_request, file_path).await;
        let response = format!(
            "User {} was successfully created/updated into bootstrap node\n",
            &user.uid
        );

        Ok(Response::new(CreateUsersResponse { response }))
    }
    async fn update_users(
        &self,
        request: Request<UpdateUsersRequest>,
    ) -> AuctionResult<UpdateUsersResponse> {
        let bid_str_request = request.into_inner().clone().bid_str;

        let bid: Bid = serde_json::from_str(&bid_str_request).unwrap();
        let file_path = format!("users/{}.json", bid.bidder);

        let mut user: User = load_users_from_file(&file_path)
            .await
            .expect("error reading user from file");

        user.credits = user.credits - bid.amount;
        user.auctions_winner.push(bid.auction_signature);
        let user_json = serde_json::to_string_pretty(&user).expect("Failed to serialize users");

        save_user_in_file(&user_json, file_path).await;

        let response = format!(
            "User {} was successfully created/updated into bootstrap node\n",
            &user.uid
        );

        Ok(Response::new(UpdateUsersResponse { response }))
    }
    async fn get_users(
        &self,
        request: Request<GetUsersRequest>,
    ) -> AuctionResult<GetUsersResponse> {
        let id = request.into_inner().clone().id;

        let file_path = format!("users/{}.json", id);

        let user = fs::read_to_string(file_path).unwrap();

        Ok(Response::new(GetUsersResponse { user }))
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
