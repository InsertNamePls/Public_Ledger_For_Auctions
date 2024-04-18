use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use std::{fs, io};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bid {
    pub auction_id: u32,
    pub bidder: String,
    pub amount: f32,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auction {
    pub auction_id: u32,
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
        auction_id: u32,
        item_name: String,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        starting_bid: f32,
        user_id: String,
        signature: String,
    ) -> Self {
        Auction {
            auction_id,
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
    pub auctions: Vec<Auction>,
}

impl AuctionHouse {
    pub fn new() -> Self {
        AuctionHouse {
            auctions: Vec::<Auction>::new(),
        }
    }

    pub fn add_auction(&mut self, auction: Auction) {
        self.auctions.push(auction);
    }
}
pub fn save_auction_data(
    auctions: &AuctionHouse,
    ip_addr: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = serde_json::to_string_pretty(&auctions)?;
    fs::write(
        format!("auctions/auction_data_{}.json", ip_addr),
        serialized,
    )
    .expect("error writing action data");
    Ok(())
}

pub async fn load_auction_data() -> Result<AuctionHouse, Box<dyn std::error::Error>> {
    let data = fs::read_to_string("/auctions/auction_data.json")?;
    let auctions: AuctionHouse = serde_json::from_str(&data)?;
    Ok(auctions)
}
pub async fn list_auctions() -> AuctionHouse {
    let result = get_files_in_directory("auctions");
    match result {
        Ok(n) => {
            let auction_house = build_auctions_from_files(&n).await;
            let mut bidding_price;
            for auction in auction_house.auctions.iter() {
                if auction.bids.is_empty() {
                    bidding_price = auction.starting_bid;
                } else {
                    bidding_price = auction.bids[auction.bids.len() - 1].amount;
                }

                println!(
                    "id: {} auction_name: {}, end_time:{}, biding_price: {:?}, auction_state:{}",
                    auction.auction_id,
                    auction.item_name,
                    auction.end_time,
                    bidding_price,
                    auction.active
                )
            }
            auction_house
        }
        Err(_e) => {
            println!("No auctions available!");
            AuctionHouse::new()
        }
    }
}

async fn build_auctions_from_files(files: &Vec<std::string::String>) -> AuctionHouse {
    let mut major_auction = AuctionHouse::new();
    for file in files {
        let data = fs::read_to_string(format!("auctions/{}", file)).unwrap();
        let resudual_auction_house: AuctionHouse =
            serde_json::from_str(&data).expect("Failed to deserialize JSON");
        for auction in resudual_auction_house.auctions.iter() {
            major_auction.add_auction(auction.to_owned());
        }
    }
    major_auction
}

pub fn get_files_in_directory(path: &str) -> io::Result<Vec<String>> {
    let entries = fs::read_dir(path)?;

    let file_names: Vec<String> = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() {
                path.file_name()?.to_str().map(|s| s.to_owned())
            } else {
                None
            }
        })
        .collect();

    Ok(file_names)
}
