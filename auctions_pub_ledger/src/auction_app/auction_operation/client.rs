use crate::auction_app::auction::{save_auction_data, Auction, AuctionHouse, Bid, Transaction};
use crate::auction_app::user::{save_user_in_file, User};
use crate::auction_tx::{
    auction_tx_client::AuctionTxClient, CreateUsersRequest, GetAuctionsRequest, GetUsersRequest,
    SubmitTransactionRequest, UpdateUsersRequest,
};
use serde::{Deserialize, Serialize};
use tonic::transport::{Certificate, Channel, ClientTlsConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub subscriber_addrs: String,
    pub transaction: Transaction,
}
// GRPC auction client
pub async fn run_client(
    dest_addr: &str,
) -> Result<AuctionTxClient<Channel>, Box<dyn std::error::Error>> {
    let ca = std::fs::read_to_string("tls/rootCA.crt")?;
    let tls = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(ca))
        .domain_name("auctiondht.fc.up.pt");
    let channel = Channel::builder(format!("https://{}:3000", dest_addr).parse().unwrap())
        .tls_config(tls)
        .unwrap()
        .connect()
        .await
        .unwrap();

    let client = AuctionTxClient::new(channel);

    Ok(client)
}
pub async fn run_client_async(
    dest_addr: String,
) -> Result<AuctionTxClient<Channel>, Box<dyn std::error::Error + Send + Sync>> {
    let ca = std::fs::read_to_string("tls/rootCA.crt")?;
    let tls = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(ca))
        .domain_name("auctiondht.fc.up.pt");
    let channel = Channel::builder(format!("https://{}:3000", dest_addr).parse().unwrap())
        .tls_config(tls)
        .unwrap()
        .connect()
        .await
        .unwrap();

    let client = AuctionTxClient::new(channel);

    Ok(client)
}
pub async fn send_transaction(
    data: Transaction,
    dest_addr: &str,
    subscriber_addr: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = run_client(dest_addr).await?;

    let transation_info: TransactionInfo = TransactionInfo {
        subscriber_addrs: subscriber_addr,
        transaction: data,
    };

    let data_str = serde_json::to_string(&transation_info).unwrap();
    let request = tonic::Request::new(SubmitTransactionRequest {
        transaction: data_str,
    });

    let response = client.submit_transaction(request).await?;

    Ok(response.into_inner().message)
}

pub async fn get_auction_house(peers: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    for peer in peers {
        let mut client = run_client(peer).await?;

        let request = tonic::Request::new(GetAuctionsRequest {});
        let response = client.get_auctions(request).await?;

        let mut auctionshouse: AuctionHouse = AuctionHouse::new();

        let auctions_vector_str: Vec<String> =
            serde_json::from_str(&response.into_inner().auctions)?;

        for auction_str in auctions_vector_str {
            let auction: Auction = serde_json::from_str(&auction_str).unwrap();

            auctionshouse.auctions.push(auction);
        }
        save_auction_data(&auctionshouse, &peer.clone())?;
    }
    //
    Ok(())
}

pub async fn create_user(peer: &str, user_str: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = run_client(peer).await?;

    let user = user_str.to_owned();

    let request = tonic::Request::new(CreateUsersRequest { user });
    let response = client.create_users(request).await?;

    Ok(response.into_inner().response)
}

pub async fn update_user(peer: &str, bid: &Bid) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = run_client(peer).await?;

    let bid_str = serde_json::to_string(bid).unwrap().to_owned();

    let request = tonic::Request::new(UpdateUsersRequest { bid_str });
    let response = client.update_users(request).await?;

    Ok(response.into_inner().response)
}

pub async fn get_user(peer: &str, id: &str) -> Result<User, Box<dyn std::error::Error>> {
    let mut client = run_client(peer).await?;

    let id = id.to_owned();

    let request = tonic::Request::new(GetUsersRequest { id });
    let response = client.get_users(request).await?.into_inner().user;
    let user: User = serde_json::from_str(&response).unwrap();
    let file_path = format!("users/{}   .json", user.user_name);

    save_user_in_file(&response, file_path).await;

    Ok(user)
}
