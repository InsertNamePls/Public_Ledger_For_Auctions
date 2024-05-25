use crate::auction_app::auction::Bid;
use crate::notification_tx::notification_tx_client::NotificationTxClient;
use crate::notification_tx::SendNotificationRequest;
use tonic::transport::Certificate;
use tonic::transport::Channel;
use tonic::transport::ClientTlsConfig;
pub async fn send_notification(
    dest_addr: String,
    data: Bid,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let data_str = serde_json::to_string(&data).unwrap();
    let mut client = run_client_async(dest_addr).await?;

    let request = tonic::Request::new(SendNotificationRequest {
        notification: data_str,
    });
    let response = client.send_notification(request).await?;
    let result_response = response.into_inner().resp;

    Ok(result_response)
}
pub async fn run_client_async(
    dest_addr: String,
) -> Result<NotificationTxClient<Channel>, Box<dyn std::error::Error + Send + Sync>> {
    let ca = std::fs::read_to_string("tls/rootCA.crt")?;
    let tls = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(ca))
        .domain_name("auctiondht.fc.up.pt");
    let channel = Channel::builder(format!("https://{}:3002", dest_addr).parse().unwrap())
        .tls_config(tls)
        .unwrap()
        .connect()
        .await
        .unwrap();

    let client = NotificationTxClient::new(channel);

    Ok(client)
}
