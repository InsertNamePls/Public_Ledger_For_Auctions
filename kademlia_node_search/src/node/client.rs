use std::time::{SystemTime, UNIX_EPOCH};
use tonic:: Status;
use crate::kademlia::{PingRequest, PingResponse};
use crate::node::crypto::Crypto;
use ring::signature::{Ed25519KeyPair, KeyPair};

pub struct Client;

impl Client {
    pub fn create_ping_request(keypair: &Ed25519KeyPair, addr: String) -> PingRequest {
        let node_address = addr;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let message = format!("{}{}", node_address, timestamp).into_bytes();
        let signature = Crypto::sign_message(keypair, &message);
        let sender_public_key = keypair.public_key().as_ref().to_vec();

        PingRequest {
            node_address,
            timestamp,
            signature,
            sender_public_key,
        }
    }

    pub async fn send_ping_request(keypair: &Ed25519KeyPair, addr: String, server_addr: String) -> Result<PingResponse, Status> {
        let endpoint = format!("http://{}", server_addr);
        /*let channel = Channel::from_shared(endpoint)?.connect().await?;
        let mut client = KademliaClient::new(channel);

        let ping_request = Self::create_ping_request(keypair, addr);
        let request = Request::new(ping_request);
        let response = client.ping(request).await?.into_inner();
        Ok(response)

        */
        unimplemented!()
    }
}
