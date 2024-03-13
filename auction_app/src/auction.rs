use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bid {
    pub bidder: String,
    pub amount: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auction {
    pub id: u32,
    pub item_name: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub start_time: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub end_time: DateTime<Utc>,
    pub starting_bid: f32,
    pub bids: Vec<Bid>,
    pub active: bool,
    pub user_id: String,
}

impl Auction {
    pub fn new(
        id: u32,
        item_name: String,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        starting_bid: f32,
        user_id: String,
    ) -> Self {
        Auction {
            id,
            item_name,
            start_time,
            end_time,
            starting_bid,
            bids: Vec::new(),
            active: true,
            user_id,
        }
    }

    pub fn place_bid(&mut self, bidder: String, amount: f32) -> Result<(), &'static str> {
        if !self.active || Utc::now() > self.end_time {
            return Err("Auction is not active or has ended.");
        }
        if let Some(last_bid) = self.bids.last() {
            if amount <= last_bid.amount {
                return Err("Bid must be higher than the current highest bid.");
            }
        } else if amount < self.starting_bid {
            return Err("Bid must be higher than the starting bid.");
        }
        self.bids.push(Bid { bidder, amount });
        Ok(())
    }

    pub fn close_auction(&mut self) -> Option<&Bid> {
        self.active = false;
        self.bids.last()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuctionHouse {
    pub auctions: HashMap<u32, Auction>,
}

impl AuctionHouse {
    pub fn new() -> Self {
        AuctionHouse {
            auctions: HashMap::new(),
        }
    }

    pub fn add_auction(&mut self, auction: Auction) {
        self.auctions.insert(auction.id, auction);
    }

    pub fn list_active_auctions(&self) {
        for auction in self.auctions.values().filter(|a| a.active) {
            println!("{:?}", auction);
        }
    }

    pub fn close_auction(&mut self, auction_id: u32) -> Result<Option<&Bid>, &'static str> {
        if let Some(auction) = self.auctions.get_mut(&auction_id) {
            if auction.active {
                let winner = auction.close_auction();
                return Ok(winner);
            }
        }
        Err("Auction not found or already closed.")
    }

    pub fn generate_auction_id(&self) -> u32 {
        self.auctions.len() as u32 + 1
    }

    pub fn place_bid(
        &mut self,
        auction_id: u32,
        bidder: String,
        amount: f32,
    ) -> Result<(), &'static str> {
        if let Some(auction) = self.auctions.get_mut(&auction_id) {
            return auction.place_bid(bidder, amount);
        }
        Err("Auction not found.")
    }
}
pub struct AuctionTransaction {
    auction_id: u32,
}
pub fn save_auction_data(auctions: &AuctionHouse) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = serde_json::to_string_pretty(&auctions)?;
    fs::write("auction_data.json", serialized)?;
    Ok(())
}

pub fn load_auction_data() -> Result<AuctionHouse, Box<dyn std::error::Error>> {
    let data = fs::read_to_string("auction_data.json")?;
    let auctions: AuctionHouse = serde_json::from_str(&data)?;
    Ok(auctions)
}

pub fn generate_initial_auction_data() -> AuctionHouse {
    let mut auction_house = AuctionHouse::new();
    let start_time = Utc::now();
    let end_time = start_time + Duration::days(1);
    let auction = Auction::new(
        auction_house.generate_auction_id(),
        "Example Item".to_string(),
        start_time,
        end_time,
        50.0,
        "example_user".to_string(),
    );
    auction_house.add_auction(auction);
    auction_house
}
