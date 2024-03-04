use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Bid {
    pub bidder: String,
    pub amount: f32,
}

#[derive(Debug, Clone)]
pub struct Auction {
    pub id: u32,
    pub item_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub starting_bid: f32,
    pub bids: Vec<Bid>,
    pub active: bool,
}

impl Auction {
    pub fn new(id: u32, item_name: String, start_time: DateTime<Utc>, end_time: DateTime<Utc>, starting_bid: f32) -> Self {
        Auction {
            id,
            item_name,
            start_time,
            end_time,
            starting_bid,
            bids: Vec::new(),
            active: true,
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

#[derive(Debug)]
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

    // Method to generate a unique auction ID (simplified version)
    pub fn generate_auction_id(&self) -> u32 {
        self.auctions.len() as u32 + 1
    }

    // Method to get the latest auction ID
    pub fn get_latest_auction_id(&self) -> u32 {
        self.auctions.len() as u32
    }

    // Wrapper around Auction's place_bid method to integrate with AuctionHouse
    pub fn place_bid(&mut self, auction_id: u32, bidder: String, amount: f32) -> Result<(), &'static str> {
        if let Some(auction) = self.auctions.get_mut(&auction_id) {
            return auction.place_bid(bidder, amount);
        }
        Err("Auction not found.")
    }
}
