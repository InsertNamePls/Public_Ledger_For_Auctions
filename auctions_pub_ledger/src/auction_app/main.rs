mod auction;
mod user;
use crate::auction::{list_auctions, Transaction};
use crate::auction::{Auction, Bid};
use auction_client::{get_auction_house, send_transaction};
use chrono::Duration;
use chrono::Utc;
use k256::ecdsa::SigningKey;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use user::User;
mod auction_client;
use sha256::digest;

#[path = "../cryptography/ecdsa_keys.rs"]
mod keys;
use crate::keys::{generate_ecdsa_keypair, load_ecdsa_keys};
use k256::ecdsa::{signature::Signer, Signature};
#[cfg(not(target_os = "windows"))]
fn clear_screen() {
    std::process::Command::new("clear").status().unwrap();
}

#[tokio::main]
async fn main() {
    clear_screen();
    let args: Vec<String> = env::args().collect();
    println!("Welcome to the BidBuddie's Auction System!");

    println!("Please select an option:\n1. Login\n2. Register");
    let mut option = String::new();
    io::stdin()
        .read_line(&mut option)
        .expect("Failed to read line");

    let mut user = match option.trim() {
        "1" => login(),
        "2" => register_user().await,
        _ => {
            println!("Invalid option, please try again.");
            return;
        }
    };

    loop {
        clear_screen();
        println!("=== Main Menu ===");
        println!("1. Auctions");
        println!("2. Profile");
        println!("3. Exit");
        print!("Select an option: ");
        io::stdout().flush().unwrap();

        let mut option = String::new();
        io::stdin()
            .read_line(&mut option)
            .expect("Failed to read line");

        match option.trim() {
            "1" => auctions_menu(&mut user, args[1].split(",").map(String::from).collect()).await,
            "2" => profile_menu(&mut user).await,
            "3" => {
                println!("Exiting...");
                break;
            }
            _ => {
                println!("Invalid option, please try again.");
            }
        }
    }
}

fn login() -> User {
    println!("Please enter your Username:");
    let mut username = String::new();
    io::stdin()
        .read_line(&mut username)
        .expect("Failed to read line");
    let username = username.trim();

    // Load users from "users.json"
    let users = load_users_from_file("users.json").expect("Failed to load users");

    //
    // Attempt to find the user with the given uid and ssh_key_path

    for user in users {
        if user.user_name == username {
            return user;
        }
    }
    clear_screen();

    println!("Login failed. User don't exist!!!");
    login() // Retry login
}

fn load_users_from_file(file_path: &str) -> Result<Vec<User>, serde_json::Error> {
    let data = match fs::read_to_string(file_path) {
        Ok(data) => data,
        Err(_) => {
            println!("No existing users found or unable to read the file.");
            return Ok(vec![]);
        }
    };
    serde_json::from_str(&data)
}

async fn register_user() -> User {
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
        participated_auctions: HashMap::new(),
    };

    // Path to the JSON file where users are stored
    let file_path = "users.json";
    let mut users = Vec::new();

    // Add the new user to the vector and write it back to the file
    users.push(user.clone()); // Clone user for push to avoid move
    let users_json = serde_json::to_string_pretty(&users).expect("Failed to serialize users");
    fs::write(file_path, users_json).expect("Failed to write to users.json");

    println!("User registered successfully.");
    user // Return the original user, not moved thanks to clone
}

async fn auctions_menu(user: &mut User, dest_ip: Vec<String>) {
    let (private_key, _) = load_ecdsa_keys(user.uid.clone());
    loop {
        clear_screen();
        println!("=== Auctions Menu ===");
        println!("1. Join Auction");
        println!("2. Create Auction");
        println!("3. Current Auctions");
        println!("4. History");
        println!("5. Back");
        print!("Select an option: ");
        io::stdout().flush().unwrap();

        let mut option = String::new();
        io::stdin()
            .read_line(&mut option)
            .expect("Failed to read line");

        match option.trim() {
            "1" => join_auction(user, &dest_ip, private_key.clone()).await,
            "2" => create_auction(&user, &dest_ip, private_key.clone()).await,
            "3" => current_auctions(&dest_ip).await,
            "4" => history(user),
            "5" => break,
            _ => {
                println!("Invalid option, please try again.");
            }
        }
    }
}

async fn profile_menu(user: &mut User) {
    loop {
        clear_screen();
        println!("=== Profile Menu ===");
        println!("1. View Profile");
        println!("2. Add Credits");
        println!("3. History");
        println!("4. Back");
        print!("Select an option: ");
        io::stdout().flush().unwrap();

        let mut option = String::new();
        io::stdin()
            .read_line(&mut option)
            .expect("Failed to read line");

        match option.trim() {
            "1" => view_profile(user),
            "2" => add_credits(user).await,
            "3" => history(user),
            "4" => break,
            _ => {
                println!("Invalid option, please try again.");
            }
        }
    }
}

fn view_profile(user: &User) {
    clear_screen();
    println!("User Profile:");
    println!("Uid: {}", user.uid);
    println!("Username: {}", user.user_name);
    println!("Credits: ${}", user.credits);

    // Displaying the user's participated auctions:
    println!("Participated Auctions:");
    user.list_participated_auctions();

    pause();
}

async fn add_credits(user: &mut User) {
    clear_screen();
    println!("Adding credits to your account.");
    println!("Enter the amount you want to add (e.g., 100):");
    let mut amount_str = String::new();
    io::stdin().read_line(&mut amount_str).unwrap();
    // Skipping input validation for simplicity
    let amount: f32 = amount_str.trim().parse().unwrap();
    user.add_credits(amount);

    println!(
        "Credits added successfully! Your new balance is ${}",
        user.credits
    );
    pause();
    // Implementation for adding credits to the user's account
}

async fn join_auction(user: &mut User, dest_ip: &Vec<String>, private_key: SigningKey) {
    clear_screen();

    get_auction_house(dest_ip)
        .await
        .expect("error geting acution from peers");

    list_auctions().await;
    let auction_id;
    loop {
        println!("Enter the Auction ID you want to join (or 'exit' to cancel):");
        let mut auction_id_str = String::new();
        io::stdin()
            .read_line(&mut auction_id_str)
            .expect("Failed to read line");

        // Trim the input and check if the user wants to exit this prompt
        let trimmed_input = auction_id_str.trim();
        if trimmed_input.eq_ignore_ascii_case("exit") {
            return;
        }

        // Attempt to parse the input as an integer
        match trimmed_input.parse::<u32>() {
            Ok(id) => {
                auction_id = id;
                break; // Exit the loop on successful parse/
            }
            Err(_) => println!("Please enter a valid ID or 'exit' to cancel."),
        }
    }

    // Proceed with the rest of the function using the valid `auction_id`
    println!("Your balance: ${}", user.credits);
    println!("Enter your bid amount:");
    let mut amount_str = String::new();
    io::stdin().read_line(&mut amount_str).unwrap();
    let amount: f32 = match amount_str.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Please enter a valid bid amount.");
            return;
        }
    };

    let signed_content = digest(auction_id.to_string() + &user.uid.clone() + &amount.to_string());
    let signature: Signature = private_key.sign(signed_content.as_bytes());
    if user.credits >= amount {
        let bid = Bid {
            auction_id,
            bidder: user.uid.clone(),
            amount,
            signature: hex::encode(signature.to_bytes()),
        };

        match send_transaction(Transaction::Bid(bid.clone()), dest_ip[0].clone()).await {
            Ok(result) => {
                println!("Transaction generated -> {:?} ", result);
            }
            Err(e) => {
                println!("error {}", e);
            }
        }
    } else {
        println!("Insufficient credits to place bid.");
    }
    pause();
}

async fn create_auction(user: &User, dest_ip: &Vec<String>, private_key: SigningKey) {
    clear_screen();
    println!("Creating a new auction.");
    println!("Enter the item name:");
    let mut item_name = String::new();
    io::stdin()
        .read_line(&mut item_name)
        .expect("Failed to read line");

    println!("Enter the starting bid:");
    let mut starting_bid_str = String::new();
    io::stdin()
        .read_line(&mut starting_bid_str)
        .expect("Failed to read line");
    let starting_bid: f32 = starting_bid_str
        .trim()
        .parse()
        .expect("Please enter a valid number");

    let start_time = Utc::now();
    println!("Enter the auction duration in days:");
    let mut duration_str = String::new();
    io::stdin()
        .read_line(&mut duration_str)
        .expect("Failed to read line");
    let duration: i64 = duration_str
        .trim()
        .parse()
        .expect("Please enter a valid number of days");
    let end_time = start_time + Duration::minutes(duration);

    get_auction_house(dest_ip)
        .await
        .expect("error geting acution from peers");

    let auction_house = list_auctions().await;
    println!("local auction house {:?}\n\n", auction_house.clone());

    let signed_content = digest(
        auction_house.auctions.len().to_string()
            + item_name.trim()
            + &starting_bid.to_string()
            + &user.uid.clone().to_string(),
    );
    let signature: Signature = private_key.sign(signed_content.as_bytes());

    // Use user.uid to pass the creator's uid to the new auction
    let auction = Auction::new(
        auction_house.auctions.len() as u32,
        item_name.trim().to_string(),
        start_time,
        end_time,
        starting_bid,
        user.uid.clone(), // Pass the user's uid as the creator
        hex::encode(signature.to_bytes()),
    );
    println!("{:?}", auction.clone());

    send_transaction(Transaction::Auction(auction.clone()), dest_ip[0].clone())
        .await
        .expect("ERROR sending transaction");
    println!("Auction created successfully!");
    pause();
}

async fn current_auctions(dest_ip: &Vec<String>) {
    clear_screen();

    println!("Active Auctions:");

    get_auction_house(dest_ip)
        .await
        .expect("error geting auction from peers");

    list_auctions().await;
    pause();
}

fn history(user: &User) {
    clear_screen();
    println!("Participated Auctions:");
    user.list_participated_auctions();

    pause();
}

fn pause() {
    let mut pause = String::new();
    println!("\nPress Enter to continue...");
    io::stdin().read_line(&mut pause).unwrap();
}
