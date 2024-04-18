use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self};

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
}

impl User {
    pub fn new(user_name: String, uid: String) -> Self {
        User {
            uid,
            user_name,
            credits: 0.0,
            participated_auctions: HashMap::new(),
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
}
