use super::auction::*;
use crate::auction_client::send_transaction;
use chrono::Utc;
use elliptic_curve::generic_array::GenericArray;
use k256::ecdsa::Signature;
use k256::ecdsa::{signature::Verifier, VerifyingKey};
use sha256::digest;
use std::env;
use std::fs;

pub async fn transaction_handler(tx: String) {
    let transaction: Transaction = serde_json::from_str(&tx).unwrap();
    let data = fs::read_to_string("auction_data.json").expect("Unable to read file");
    let mut auction_house: AuctionHouse =
        serde_json::from_str(&data).expect("Failed to deserialize JSON");
    match transaction {
        Transaction::Bid(ref value) => {
            println!("\n{:?}", value);
            find_auction_to_bid(auction_house.clone(), &value, transaction.clone()).await;
        }
        Transaction::Auction(value) => {
            println!("\n{:?}", auction_house);

            let signed_content = digest(
                value.auction_id.to_string()
                    + &value.item_name
                    + &value.starting_bid.to_string()
                    + &value.user_id.to_string(),
            );
            if validate_tx_integrity(&signed_content, &value.user_id, value.signature.clone()).await
            {
                auction_house.add_auction(value);
                let serialized = serde_json::to_string_pretty(&auction_house).unwrap();
                fs::write("auction_data.json", serialized).expect("error saving auction");
            }
        }
    }
}
pub async fn validate_tx_integrity(
    signed_content: &String,
    uid: &String,
    sig_string: String,
) -> bool {
    // get puiblic key from uid hex value
    let public_key = VerifyingKey::from_sec1_bytes(&hex::decode(uid).unwrap())
        .expect("error converting uid to public key");

    let sig: Signature = Signature::from_bytes(&GenericArray::clone_from_slice(
        &hex::decode(sig_string).unwrap(),
    ))
    .expect("error decoding signature");

    // validate signature with the concat of parameters
    public_key.verify(signed_content.as_bytes(), &sig).is_ok()
}

pub async fn find_auction_to_bid(
    mut auction_house: AuctionHouse,
    bid: &Bid,
    transaction: Transaction,
) {
    if let Some(auction) = auction_house
        .auctions
        .iter()
        .find(|auction| auction.auction_id == bid.auction_id)
    {
        let signed_content =
            digest(bid.auction_id.to_string() + &bid.bidder + &bid.amount.to_string());
        println!("{:?}\n", signed_content);
        println!("{:?}\n", bid.signature.clone());

        if validate_tx_integrity(&signed_content, &bid.bidder, bid.signature.clone()).await {
            auction_house = bid_handler(auction.clone(), auction_house.clone(), bid.clone()).await;
            let serialized = serde_json::to_string_pretty(&auction_house).unwrap();
            fs::write("auction_data.json", serialized).expect("error saving bid");
        }
    } else {
        println!("Auction not present, sending to other peers");
        //
        let dest_ip = env::var("NGH_ADDR").unwrap();
        println!("{:?}", dest_ip);
        match send_transaction(transaction, dest_ip).await {
            Ok(result) => {
                println!("Transaction generated -> {:?} ", result);
            }
            Err(e) => {
                println!("error {}", e);
            }
        }
    }
}

pub async fn bid_handler(
    target_auction: Auction,
    mut auction_house: AuctionHouse,
    bid: Bid,
) -> AuctionHouse {
    let mut last_highest_bid = 0.0;
    if !target_auction.bids.is_empty() {
        last_highest_bid = target_auction.bids[target_auction.bids.len() - 1].amount;
    }

    if &target_auction.end_time > &Utc::now() && last_highest_bid < bid.amount {
        let target_auction_position = auction_house
            .auctions
            .iter()
            .position(|auction| auction.auction_id == target_auction.auction_id)
            .unwrap();

        auction_house.auctions[target_auction_position]
            .bids
            .push(bid);

        let serialized = serde_json::to_string_pretty(&auction_house).unwrap();
        fs::write("auction_data.json", serialized).expect("error writing auction data");
        auction_house
    } else {
        auction_house
    }
}

// GRPC auction server
use auction_tx::auction_tx_server::AuctionTxServer;
use auction_tx::{
    GetAuctionsRequest, GetAuctionsResponse, SubmitTransactionRequest, SubmitTransactionResponse,
};
use tonic::{
    transport::{Identity, Server, ServerTlsConfig},
    Request, Response, Status,
};
pub mod auction_tx {
    tonic::include_proto!("auction_tx");
}

type AuctionResult<T> = Result<Response<T>, Status>;

#[derive(Default)]
pub struct AuctionsTxServer {}

#[tonic::async_trait]
impl auction_tx::auction_tx_server::AuctionTx for AuctionsTxServer {
    async fn submit_transaction(
        &self,
        request: Request<SubmitTransactionRequest>,
    ) -> AuctionResult<SubmitTransactionResponse> {
        let message = request.into_inner().transaction;
        transaction_handler(message.clone()).await;

        Ok(Response::new(SubmitTransactionResponse { message }))
    }
    async fn get_auctions(
        &self,
        _: Request<GetAuctionsRequest>,
    ) -> AuctionResult<GetAuctionsResponse> {
        let local_auction_house =
            fs::read_to_string("auction_data.json").expect("Unable to read file");
        Ok(Response::new(GetAuctionsResponse {
            auctions: local_auction_house,
        }))
    }
}
pub async fn auction_server() {
    let cert = std::fs::read_to_string("tls/server.crt");
    let key = std::fs::read_to_string("tls/server.key");

    let identity = Identity::from_pem(cert.unwrap(), key.unwrap());

    let addr = "0.0.0.0:3000".parse().unwrap();
    let auction_svr = AuctionsTxServer::default();
    //::default();
    println!("Greet server listening on {}", addr);
    Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))
        .unwrap()
        .add_service(AuctionTxServer::new(auction_svr))
        .serve(addr)
        .await
        .expect("error building server");
}
