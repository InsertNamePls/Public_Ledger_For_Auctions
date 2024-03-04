mod user;
mod auction;

use user::User;
use auction::AuctionHouse;
use std::io::{self, Write};
use chrono::Utc;

#[cfg(target_os = "windows")]
fn clear_screen() {
    std::process::Command::new("cmd").args(&["/C", "cls"]).status().unwrap();
}

#[cfg(not(target_os = "windows"))]
fn clear_screen() {
    std::process::Command::new("clear").status().unwrap();
}

fn main() {
    let mut auction_house = AuctionHouse::new();
    clear_screen();
    println!("Welcome to the BidBuddie's Auction System!");
    println!("Enter your username:");
    let mut username = String::new();
    io::stdin().read_line(&mut username).expect("Failed to read line");
    let mut user = User::new(username.trim().to_string());

    loop {
        clear_screen();
        println!("=== Main Menu ===");
        println!("1. Auctions");
        println!("2. Profile");
        println!("3. Exit");
        print!("Select an option: ");
        io::stdout().flush().unwrap();

        let mut option = String::new();
        io::stdin().read_line(&mut option).expect("Failed to read line");

        match option.trim() {
            "1" => auctions_menu(&mut user, &mut auction_house),
            "2" => profile_menu(&mut user),
            "3" => {
                println!("Exiting...");
                break;
            },
            _ => {
                println!("Invalid option, please try again.");
            },
        }
    }
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
        io::stdin().read_line(&mut option).expect("Failed to read line");

        match option.trim() {
            "1" => join_auction(user, auction_house),
            "2" => create_auction(user, auction_house),
            "3" => current_auctions(user, auction_house),
            "4" => history(user),
            "5" => break,
            _ => {
                println!("Invalid option, please try again.");
            },
        }
    }
}

fn profile_menu(user: &mut User) {
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
        io::stdin().read_line(&mut option).expect("Failed to read line");

        match option.trim() {
            "1" => view_profile(user),
            "2" => add_credits(user),
            "3" => history(user),
            "4" => break,
            _ => {
                println!("Invalid option, please try again.");
            },
        }
    }
}

fn view_profile(user: &User) {
    clear_screen();
    println!("User Profile:");
    println!("Username: {}", user.identifier);
    println!("Credits: ${}", user.credits);
    user.list_participated_auctions();

    pause();

    // Implementation for viewing the user profile
}

fn add_credits(user: &mut User) {
    clear_screen();
    println!("Adding credits to your account.");
    println!("Enter the amount you want to add (e.g., 100):");
    let mut amount_str = String::new();
    io::stdin().read_line(&mut amount_str).unwrap();
    // Skipping input validation for simplicity
    let amount: f32 = amount_str.trim().parse().unwrap();
    user.add_credits(amount);
    println!("Credits added successfully! Your new balance is ${}", user.credits);
    pause();
    // Implementation for adding credits to the user's account
}

fn join_auction(user: &mut User, auction_house: &AuctionHouse) {
    clear_screen();
    println!("Active Auctions:");
    // Assuming list_active_auctions just prints out auctions for now
    auction_house.list_active_auctions();
    println!("(Template) Enter the Auction ID you want to join (e.g., 1):");
    let mut auction_id_str = String::new();
    io::stdin().read_line(&mut auction_id_str).unwrap();
    // Skipping input validation for simplicity
    println!("(Template) Enter your bid amount (e.g., 100):");
    // Skipping bid placement since it involves more complex logic
    println!("(Template) Bid placed successfully for Auction ID {}", auction_id_str.trim());
    pause();
}

fn create_auction(user: &mut User, auction_house: &mut AuctionHouse) {
    clear_screen();
    println!("(Template) Creating a new auction with predefined values.");
    // Example values
    let item_name = "Test Item";
    let starting_bid: f32 = 50.0;
    // Normally, you'd capture user input here
    println!("Auction for '{}' with starting bid of ${} created successfully!", item_name, starting_bid);
    pause();
}

fn current_auctions(user: &User, auction_house: &AuctionHouse) {
    clear_screen();
    println!("(Template) Your Current Auctions:");
    // Display some template data
    println!("Auction ID: 1, Item: Test Item, Bid: $100");
    pause();
}

fn history(user: &User) {
    clear_screen();
    println!("(Template) Auction History:");
    // Display some template history data
    println!("Auction ID: 1, Item: Old Item, Your Bid: $90, Status: Won");
    pause();
}


fn pause() {
    let mut pause = String::new();
    println!("\nPress Enter to continue...");
    io::stdin().read_line(&mut pause).unwrap();
}
