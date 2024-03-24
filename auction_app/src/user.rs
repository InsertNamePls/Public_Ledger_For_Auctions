use crate::auction::{Auction, AuctionHouse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

// The AuctionActivity enum is used to store the activities of the user in the auctions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuctionActivity {
    Created(u32),  // Contains Auction ID
    Bid(u32, f32), // Contains Auction ID and bid amount
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub uid: String,
    pub user_name: String,
    pub credits: f32,
    pub participated_auctions: HashMap<u32, Vec<AuctionActivity>>,
    pub ssh_key_path: String,
}

impl User {
    pub fn new(user_name: String, ssh_key_path: String, uid: String) -> Self {
        User {
            uid,
            user_name,
            credits: 0.0,
            participated_auctions: HashMap::new(),
            ssh_key_path,
        }
    }

    pub fn add_credits(&mut self, amount: f32) {
        self.credits += amount;
        self.update_user_file();
    }

    pub fn update_user_file(&self) {
        // The path to the 'users.json' file
        let file_path = "users.json";

        // Read the entire file and deserialize it into Vec<User>
        let file_content = fs::read_to_string(file_path).unwrap();
        let mut users: Vec<User> = serde_json::from_str(&file_content).unwrap();

        // Find the user in the vector and update their information
        if let Some(user) = users.iter_mut().find(|u| u.uid == self.uid) {
            *user = self.clone();
        }

        // Serialize the entire vector back to JSON
        let updated_content = serde_json::to_string_pretty(&users).unwrap();

        // Write the updated JSON back to the file, replacing the old content
        fs::write(file_path, updated_content).unwrap();
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

    pub fn provide_ssh_key(&self) -> std::io::Result<String> {
        // Reading the SSH public key from the provided path
        let ssh_key = fs::read_to_string(&self.ssh_key_path)?;
        Ok(ssh_key)
    }

    pub fn place_bid_with_auction_house(
        &mut self,
        auction_house: &mut AuctionHouse,
        auction_id: u32,
        bid_amount: f32,
        signature: String,
    ) -> Result<(), &'static str> {
        if self.credits < bid_amount {
            return Err("Insufficient credits.");
        }
        match auction_house.place_bid(auction_id, self.uid.clone(), bid_amount, signature) {
            Ok(()) => {
                self.credits -= bid_amount;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    // pub fn store_ssh_key(&self) -> std::io::Result<()> {
    //     // Ensure the "public_ssh_key" directory exists
    //     let dir_path = "public_ssh_key";
    //     fs::create_dir_all(dir_path)?;
    //
    //     // Reading the SSH public key from the provided path
    //     let ssh_key = fs::read_to_string(&self.ssh_key_path)?;
    //
    //     // Creating a file name based on the user uid
    //     let file_name = format!("{}/{}-ssh_key.pub", dir_path, self.user_name);
    //
    //     // Creating and writing the SSH public key to the file
    //     let mut file = File::create(Path::new(&file_name))?;
    //     file.write_all(ssh_key.as_bytes())?;
    //     Ok(())
    // }
}
