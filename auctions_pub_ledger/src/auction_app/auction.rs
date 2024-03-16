use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bid {
    pub auction_id: u32,
    pub bidder: String,
    pub amount: f32,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auction {
    pub item_name: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub start_time: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub end_time: DateTime<Utc>,
    pub starting_bid: f32,
    pub bids: Vec<Bid>,
    pub active: bool,
    pub user_id: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Transaction {
    Auction(Auction),
    Bid(Bid),
}

impl Auction {
    pub fn new(
        item_name: String,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        starting_bid: f32,
        user_id: String,
        signature: String,
    ) -> Self {
        Auction {
            item_name,
            start_time,
            end_time,
            starting_bid,
            bids: Vec::new(),
            active: true,
            user_id,
            signature,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuctionHouse {
    pub auctions: HashMap<u32, Auction>,
}

impl AuctionHouse {
    pub fn new() -> Self {
        AuctionHouse {
            auctions: HashMap::new(),
        }
    }

    pub fn add_auction(&mut self, auction: Auction, id: u32) {
        self.auctions.insert(id, auction);
    }
}
pub fn save_auction_data(auctions: &AuctionHouse) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = serde_json::to_string_pretty(&auctions)?;
    fs::write("auction_data.json", serialized);
    Ok(())
}

pub async fn load_auction_data() -> Result<AuctionHouse, Box<dyn std::error::Error>> {
    let data = fs::read_to_string("auction_data.json")?;
    let auctions: AuctionHouse = serde_json::from_str(&data)?;
    Ok(auctions)
}
pub fn list_auctions() {
    let data = fs::read_to_string("auction_data.json").expect("Unable to read file");
    let auction_house: AuctionHouse =
        serde_json::from_str(&data).expect("Failed to deserialize JSON");
    let mut bidding_price = 0.0;
    for (auction_id, auction) in auction_house.auctions.iter() {
        if auction.bids.is_empty() {
            bidding_price = auction.starting_bid;
        } else {
            bidding_price = auction.bids[auction.bids.len() - 1].amount;
        }

        println!(
            "id: {} auction_name: {}, end_time:{}, biding_price: {:?}",
            auction_id, auction.item_name, auction.end_time, bidding_price
        )
    }
}
