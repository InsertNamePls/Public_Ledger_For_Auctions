use crate::auction_app::auction_operation::client::create_user;
use crate::cryptography::ecdsa_keys::generate_ecdsa_keypair;
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs::{self};
use std::io::{self};
// The AuctionActivity enum is used to store the activities of the user in the auctions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    pub activity_type: String,
    pub auction_signature: String,
    pub amount: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub uid: String,
    pub user_name: String,
    pub credits: f32,
    pub auctions_winner: Vec<String>,
    pub activity: Vec<UserActivity>,
}

impl User {
    pub fn new(user_name: String, uid: String) -> Self {
        User {
            uid,
            user_name,
            credits: 0.0,
            auctions_winner: Vec::new(),
            activity: Vec::new(),
        }
    }

    pub fn add_credits(&mut self, amount: f32) {
        self.credits += amount;
    }
}

pub async fn register_user(peer: &str) -> User {
    println!("Please enter your Username:");
    let mut username = String::new();
    io::stdin()
        .read_line(&mut username)
        .expect("Failed to read line");

    let (_, public_key) = generate_ecdsa_keypair();
    let user = User {
        uid: hex::encode(public_key.to_sec1_bytes()),
        user_name: username.trim().to_string(),
        credits: 0.0,
        auctions_winner: Vec::new(),
        activity: Vec::new(),
    };

    // Path to the JSON file where users are stored
    let file_path = format!("users/{}.json", user.user_name);
    let user_json = serde_json::to_string_pretty(&user).expect("Failed to serialize users");

    save_user_in_file(&user_json, file_path).await;

    match create_user(peer, &user_json).await {
        Ok(response) => println!("{}", response.green()),
        Err(e) => println!(
            "{}",
            format!("Error registering user {}\n{}", &user.uid, e).red()
        ),
    }
    user
}
pub async fn save_user_in_file(user_json: &String, file_path: String) {
    fs::write(file_path, user_json).expect("Failed to write to users.json");
}
pub async fn load_users_from_file(file_path: &str) -> Result<User, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(file_path).unwrap();

    Ok(serde_json::from_str(&data)?)
}

pub async fn add_credits(peer: &str, user: &mut User) {
    std::process::Command::new("clear").status().unwrap();

    println!("Adding credits to your account.");
    println!("Enter the amount you want to add (e.g., 100):");
    let mut amount_str = String::new();

    io::stdin().read_line(&mut amount_str).unwrap();
    let amount: f32 = amount_str.trim().parse().unwrap();
    user.add_credits(amount);

    let user_json = serde_json::to_string_pretty(&user).expect("Failed to serialize users");
    match create_user(peer, &user_json).await {
        Ok(response) => println!("{}", response.green()),
        Err(e) => println!(
            "{}",
            format!("Error updation user {}\n{}", &user.user_name, e).red()
        ),
    }

    let mut pause_str = String::new();
    println!("\nPress Enter to continue...");
    io::stdin().read_line(&mut pause_str).unwrap();
}
