use crate::auction_app::auction::save_auction_data;
use crate::auction_app::auction::Auction;
use crate::auction_app::auction::AuctionHouse;
use crate::auction_app::auction::Transaction;
use crate::auction_tx::auction_tx_client::AuctionTxClient;
use crate::auction_tx::{GetAuctionsRequest, SubmitTransactionRequest};
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
// GRPC auction client

pub async fn run_client(
    dest_addr: String,
) -> Result<AuctionTxClient<Channel>, Box<dyn std::error::Error>> {
    let ca = std::fs::read_to_string("tls/rootCA.crt")?;
    let tls = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(ca))
        .domain_name("auctiondht.com");
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
        .domain_name("auctiondht.com");
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
    dest_addr: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let data_str = serde_json::to_string(&data).unwrap();
    let mut client = run_client(dest_addr).await?;

    let request = tonic::Request::new(SubmitTransactionRequest {
        transaction: data_str,
    });
    let response = client.submit_transaction(request).await?;

    Ok(response.into_inner().message)
}

pub async fn get_auction_house(peers: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    for peer in peers {
        let mut client = run_client(peer.clone()).await?;

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
