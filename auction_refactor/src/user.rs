use crate::auction::{Auction, AuctionHouse};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
enum AuctionActivity {
    Created(u32), // Contains Auction ID
    Bid(u32, f32), // Contains Auction ID and bid amount
}

#[derive(Debug)]
pub struct User {
    pub identifier: String, //saved the user identifier
    pub credits: f32, //saved the user credits
    pub participated_auctions: HashMap<u32, Vec<AuctionActivity>>, //saved the user participated auctions
    pub ssh_key_path: String, //Field to store the path to the SSH public key
}

impl User {
    pub fn new(identifier: String, ssh_key_path: String) -> Self {
        User {
            identifier,
            credits: 0.0,
            participated_auctions: HashMap::new(),
            ssh_key_path,
        }
    }

    pub fn add_credits(&mut self, amount: f32) {
        self.credits += amount;
    }

    pub fn register_auction_activity(&mut self, auction_id: u32, activity: AuctionActivity) {
        self.participated_auctions
            .entry(auction_id)
            .or_insert_with(Vec::new)
            .push(activity);
    }

    pub fn list_participated_auctions(&self) {
        for (auction_id, activities) in &self.participated_auctions {
            println!("Auction ID: {}", auction_id);
            for activity in activities {
                println!("  Activity: {:?}", activity);
            }
        }
    }

    pub fn create_auction(&self, auction_house: &mut AuctionHouse, item_name: String, start_time: DateTime<Utc>, end_time: DateTime<Utc>, starting_bid: f32) -> u32 {
        let auction = Auction::new(auction_house.generate_auction_id(), item_name, start_time, end_time, starting_bid);
        auction_house.add_auction(auction);
        auction_house.get_latest_auction_id()
    }

    pub fn place_bid_with_auction_house(&mut self, auction_house: &mut AuctionHouse, auction_id: u32, bid_amount: f32) -> Result<(), &'static str> {
        if self.credits < bid_amount {
            return Err("Insufficient credits.");
        }
        match auction_house.place_bid(auction_id, self.identifier.clone(), bid_amount) {
            Ok(()) => {
                self.credits -= bid_amount;
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    pub fn store_ssh_key(&self) -> std::io::Result<()> {
          // Reading the SSH public key from the provided path
          let ssh_key = std::fs::read_to_string(&self.ssh_key_path)?;
        
          // Creating a file name based on the user identifier
          let file_name = format!("{}-ssh_key.pub", self.identifier);
          
          // Creating and writing the SSH public key to the file
          let mut file = std::fs::File::create(&file_name)?;
          file.write_all(ssh_key.as_bytes())?;
          Ok(())
    }
}
