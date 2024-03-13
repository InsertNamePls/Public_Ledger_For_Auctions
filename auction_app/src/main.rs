mod auction;
mod user;

use chrono::Duration;
use std::collections::HashMap;
use user::User;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
//use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

use crate::auction::Auction;
use crate::auction::{
    generate_initial_auction_data, load_auction_data, save_auction_data, AuctionHouse,
};
mod auction_client;
use crate::auction_client::send_transaction;

use sha2::{Digest, Sha256};
#[cfg(target_os = "windows")]
fn clear_screen() {
    std::process::Command::new("cmd")
        .args(&["/C", "cls"])
        .status()
        .unwrap();
}

#[cfg(not(target_os = "windows"))]
fn clear_screen() {
    std::process::Command::new("clear").status().unwrap();
}

#[tokio::main]
async fn main() {
    // Change this for a scrapping function later on
    let mut auction_house = match load_auction_data() {
        Ok(data) => data,
        Err(_) => {
            let initial_data = generate_initial_auction_data(); // for testing purposes later change.
            save_auction_data(&initial_data).expect("Failed to save initial auction data");
            initial_data
        }
    };
    clear_screen();

    println!("Welcome to the BidBuddie's Auction System!");

    println!("Please select an option:\n1. Login\n2. Register");
    let mut option = String::new();
    io::stdin()
        .read_line(&mut option)
        .expect("Failed to read line");

    let mut user = match option.trim() {
        "1" => login(),
        "2" => register_user(),
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
            "1" => auctions_menu(&mut user, &mut auction_house),
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

    println!("Enter the path to your SSH key:");
    let mut ssh_key_path = String::new();
    io::stdin()
        .read_line(&mut ssh_key_path)
        .expect("Failed to read line");
    let ssh_key_path = ssh_key_path.trim();

    // Load users from "users.json"
    let users = load_users_from_file("users.json").expect("Failed to load users");

    let uid = gen_uid(&ssh_key_path.trim().to_string());

    println!("{}", uid);

    // Attempt to find the user with the given uid and ssh_key_path

    for user in users {
        if user.uid == uid && user.user_name == username {
            println!("Login successful for user: {}", username);
            return user;
        }
    }
    clear_screen();
    println!("Login failed. Username or SSH key path does not match.");
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

fn register_user() -> User {
    println!("Please enter your Username:");
    let mut username = String::new();
    io::stdin()
        .read_line(&mut username)
        .expect("Failed to read line");

    println!("Enter the path to your SSH key:");
    let mut ssh_key_path = String::new();
    io::stdin()
        .read_line(&mut ssh_key_path)
        .expect("Failed to read line");

    let uid = gen_uid(&ssh_key_path.trim().to_string());

    println!("{}", uid);
    let user = User {
        uid: uid,
        user_name: username.trim().to_string(),
        credits: 0.0,
        participated_auctions: HashMap::new(),
        ssh_key_path: ssh_key_path.trim().to_string(),
    };

    // Attempt to store the SSH key, reporting any errors encountered
    //match user.store_ssh_key() {
    //    Ok(_) => println!("SSH key stored successfully."),
    //    Err(e) => println!("Failed to store SSH key: {}", e),
    //}

    // Path to the JSON file where users are stored
    let file_path = "users.json";
    let mut users = Vec::new();

    // Check if file exists and read the existing users
    //if Path::new(file_path).exists() {
    //    let data = fs::read_to_string(file_path).expect("Failed to read users.json");
    //    users = serde_json::from_str(&data).expect("Failed to deserialize users");
    //}

    // Add the new user to the vector and write it back to the file
    users.push(user.clone()); // Clone user for push to avoid move
    let users_json = serde_json::to_string_pretty(&users).expect("Failed to serialize users");
    fs::write(file_path, users_json).expect("Failed to write to users.json");

    println!("User registered successfully.");
    user // Return the original user, not moved thanks to clone
}

fn auctions_menu(user: &mut User, auction_house: &mut AuctionHouse) {
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
            "1" => join_auction(user, auction_house),
            "2" => create_auction(&user, auction_house),
            "3" => current_auctions(user, auction_house),
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

    // Attempting to read the SSH public key from the provided path
    let ssh_key_result = std::fs::read_to_string(&user.ssh_key_path);

    // Handling the Result to safely access the SSH public key content
    match ssh_key_result {
        Ok(ssh_key) => {
            // If reading was successful, print the SSH public key
            println!("SSH Key Path: {}", user.ssh_key_path);
            println!("SSH Public Key Content:\n{}", ssh_key);
        }
        Err(e) => {
            // If there was an error reading the file, print an error message instead
            println!(
                "Failed to read SSH public key from '{}': {}",
                user.ssh_key_path, e
            );
        }
    }

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

    //send event to blockchain
    send_transaction(
        format!("user {} aded {}", user.uid, amount.to_string()),
        "0.0.0.0",
    )
    .await;

    println!(
        "Credits added successfully! Your new balance is ${}",
        user.credits
    );
    pause();
    // Implementation for adding credits to the user's account
}

fn join_auction(user: &mut User, auction_house: &mut AuctionHouse) {
    clear_screen();

    println!("Active Auctions:");
    auction_house.list_active_auctions();

    let mut auction_id = 0;
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
                break; // Exit the loop on successful parse
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

    if user.credits >= amount {
        if let Ok(_) = auction_house.place_bid(auction_id, user.uid.clone(), amount) {
            println!("Bid placed successfully for Auction ID {}", auction_id);
            user.credits -= amount; // Update user credits
            save_auction_data(auction_house).expect("Failed to save auction data");
        } else {
            println!("Failed to place bid. Auction might not exist or be closed.");
        }
    } else {
        println!("Insufficient credits to place bid.");
    }
    pause();
}

fn create_auction(user: &User, auction_house: &mut AuctionHouse) {
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
    let end_time = start_time + Duration::days(duration);

    // Use user.uid to pass the creator's uid to the new auction
    let auction_id = auction_house.generate_auction_id();
    let auction = Auction::new(
        auction_id,
        item_name.trim().to_string(),
        start_time,
        end_time,
        starting_bid,
        user.uid.clone(), // Pass the user's uid as the creator
    );

    auction_house.add_auction(auction);
    save_auction_data(auction_house).expect("Failed to save auction data");
    println!("Auction created successfully!");
    pause();
}

fn current_auctions(user: &User, auction_house: &AuctionHouse) {
    clear_screen();
    println!("Active Auctions:");
    auction_house.list_active_auctions();
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

fn gen_uid(ssh_key_path: &String) -> String {
    let mut hasher = Sha256::new();
    println!("{}", ssh_key_path);
    let pub_key = fs::read_to_string(&ssh_key_path).expect("Unable to read file");

    println!();
    hasher.update(pub_key);
    let result = hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    println!("{}", result);
    result
}
