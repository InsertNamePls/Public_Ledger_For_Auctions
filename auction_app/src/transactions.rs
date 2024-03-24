use chrono::{DateTime, Utc};
use core::f32;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Transaction {
    AuctionTx(AuctionTx),
    UserTx(UserTx),
    BidTx(AuctionTx),
    CreditsUpdateTx(UserTx),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserTx {
    uid: String,
    credits: f32,
    ssh_pub_key: String,
    tx_signature: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuctionTx {
    id: u32,
    item_name: String,
    auction_creator: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    start_time: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    end_time: DateTime<Utc>,
    starting_bid: f32,
    current_bidder: String,
    bid_value: f32,
    tx_signature: String,
}
impl UserTx {
    pub fn new(uid: String, credits: f32, ssh_pub_key: String, tx_signature: String) -> Self {
        UserTx {
            uid,
            credits,
            ssh_pub_key,
            tx_signature,
        }
    }
}
impl AuctionTx {
    pub fn new(
        id: u32,
        item_name: String,
        auction_creator: String,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        starting_bid: f32,
        current_bidder: String,
        bid_value: f32,
        tx_signature: String,
    ) -> Self {
        AuctionTx {
            id,
            item_name,
            auction_creator,
            start_time,
            end_time,
            starting_bid,
            current_bidder,
            bid_value,
            tx_signature,
        }
    }
}
