use super::auction::*;
use crate::auction_client::send_transaction;
use auction_tx::auction_tx_server::AuctionTxServer;
use auction_tx::{
    GetAuctionsRequest, GetAuctionsResponse, SubmitTransactionRequest, SubmitTransactionResponse,
};
use chrono::Utc;
use elliptic_curve::generic_array::GenericArray;
use k256::ecdsa::Signature;
use k256::ecdsa::{signature::Verifier, VerifyingKey};
use sha256::digest;
use std::env;
use tokio::sync::Mutex;
use tonic::{
    transport::{Identity, Server, ServerTlsConfig},
    Request, Response, Status,
};

use std::sync::Arc;
pub async fn transaction_handler(tx: String, shared_auction_house: &mut Arc<Mutex<AuctionHouse>>) {
    let transaction: Transaction = serde_json::from_str(&tx).unwrap();
    match transaction {
        Transaction::Bid(ref value) => {
            println!("\n{:?}", value);
            find_auction_to_bid(
                &mut shared_auction_house.clone(),
                value,
                transaction.clone(),
            )
            .await;
        }
        Transaction::Auction(value) => {
            println!("\n{:?}", value);
            let mut auction_house = shared_auction_house.lock().await;
            let signed_content = digest(
                value.auction_id.to_string()
                    + &value.item_name
                    + &value.starting_bid.to_string()
                    + &value.user_id.to_string(),
            );
            match validate_tx_integrity(&signed_content, &value.user_id, value.signature.clone())
                .await
            {
                Ok(_True) => {
                    auction_house.add_auction(value);
                }
                Ok(_False) => println!("signature is not valid"),
                Err(e) => println!("{:?}", e),
            }
        }
    }
}
pub async fn validate_tx_integrity(
    signed_content: &String,
    uid: &String,
    sig_string: String,
) -> Result<bool, Box<dyn std::error::Error>> {
    // get puiblic key from uid hex value
    let public_key = VerifyingKey::from_sec1_bytes(&hex::decode(uid).unwrap())?;

    let sig: Signature = Signature::from_bytes(&GenericArray::clone_from_slice(
        &hex::decode(sig_string).unwrap(),
    ))?;

    // validate signature with the concat of parameters
    Ok(public_key.verify(signed_content.as_bytes(), &sig).is_ok())
}

pub async fn find_auction_to_bid(
    shared_auction_house: &mut Arc<Mutex<AuctionHouse>>,
    bid: &Bid,
    transaction: Transaction,
) {
    let mut auction_house = shared_auction_house.lock().await;

    if let Some(auction) = auction_house
        .clone()
        .auctions
        .iter()
        .find(|auction| auction.auction_id == bid.auction_id)
    {
        let signed_content =
            digest(bid.auction_id.to_string() + &bid.bidder + &bid.amount.to_string());

        let mut last_highest_bid = 0.0;
        if !auction.clone().bids.is_empty() {
            last_highest_bid = auction.clone().bids[auction.clone().bids.len() - 1].amount;
        }

        if &auction.clone().end_time > &Utc::now() && last_highest_bid < bid.amount {
            match validate_tx_integrity(&signed_content, &bid.bidder, bid.signature.clone()).await {
                Ok(_True) => {
                    let target_auction_position = auction_house
                        .auctions
                        .iter()
                        .position(|i| i.auction_id == auction.auction_id)
                        .unwrap();

                    auction_house.auctions[target_auction_position]
                        .bids
                        .push(bid.clone());
                }
                Ok(_False) => println!("Error integrity signature is not valid"),
                Err(e) => println!("{:?}", e),
            }
        }
    } else {
        println!("Auction not present, sending to peers");
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

// GRPC auction server
pub mod auction_tx {
    tonic::include_proto!("auction_tx");
}

type AuctionResult<T> = Result<Response<T>, Status>;

#[derive(Debug, Clone)]
pub struct AuctionsTxServer {
    shared_auction_house_state: Arc<Mutex<AuctionHouse>>,
}

#[tonic::async_trait]
impl auction_tx::auction_tx_server::AuctionTx for AuctionsTxServer {
    async fn submit_transaction(
        &self,
        request: Request<SubmitTransactionRequest>,
    ) -> AuctionResult<SubmitTransactionResponse> {
        let message = request.into_inner().transaction;

        transaction_handler(
            message.clone(),
            &mut self.shared_auction_house_state.clone(),
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
pub async fn auction_server(shared_auction_house: Arc<Mutex<AuctionHouse>>) {
    let cert = std::fs::read_to_string("tls/server.crt");
    let key = std::fs::read_to_string("tls/server.key");

    let identity = Identity::from_pem(cert.unwrap(), key.unwrap());

    let addr = "0.0.0.0:3000".parse().unwrap();
    //let auction_svr = AuctionsTxServer::default();
    println!("Greet server listening on {}", addr);
    Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))
        .unwrap()
        .add_service(AuctionTxServer::new(AuctionsTxServer {
            shared_auction_house_state: shared_auction_house,
        }))
        .serve(addr)
        .await
        .expect("error building server");
}
