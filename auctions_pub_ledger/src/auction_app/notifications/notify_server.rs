use crate::auction_app::auction::Bid;
use crate::auction_app::auction::Notification;
use crate::notification_tx::notification_tx_server::NotificationTx;
use crate::notification_tx::notification_tx_server::NotificationTxServer;
use crate::notification_tx::{SendNotificationRequest, SendNotificationResponse};
use std::{fs, io};
use tonic::{
    transport::{Identity, Server, ServerTlsConfig},
    Request, Response, Status,
};
type NotificationTxResult<T> = Result<Response<T>, Status>;

#[derive(Default)]
pub struct NotificationServer {}

#[tonic::async_trait]
impl NotificationTx for NotificationServer {
    async fn send_notification(
        &self,
        request: Request<SendNotificationRequest>,
    ) -> NotificationTxResult<SendNotificationResponse> {
        let message = request.into_inner().clone().notification;
        let bid: Bid = serde_json::from_str(&message).unwrap();
        println!("{:?}", bid);
        println!(
            "New notification!\n Bid submited by: {}\n Auction: {}\n Amount: {}\n",
            bid.bidder, bid.signature, bid.amount
        );
        let notification = Notification {
            bidder: bid.bidder,
            amount: bid.amount,
            auction_signature: bid.auction_signature,
        };
        let serialized = serde_json::to_string_pretty(&notification).unwrap();

        fs::write("auction_data_notification.json", serialized).unwrap();
        let resp = "Notification submieted with success".to_string();

        Ok(Response::new(SendNotificationResponse { resp }))
    }
}
pub async fn notification_server() {
    let cert = std::fs::read_to_string("tls/server.crt");
    let key = std::fs::read_to_string("tls/server.key");

    let identity = Identity::from_pem(cert.unwrap(), key.unwrap());

    let addr = "0.0.0.0:3002".parse().unwrap();
    Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))
        .unwrap()
        .add_service(NotificationTxServer::new(NotificationServer::default()))
        .serve(addr)
        .await
        .expect("error building server");
}
