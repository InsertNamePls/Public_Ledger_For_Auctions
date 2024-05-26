use auctions_pub_ledger::auction_app::auction::list_auctions;
use auctions_pub_ledger::auction_app::auction::Auction;
use auctions_pub_ledger::auction_app::auction::Bid;
use auctions_pub_ledger::auction_app::auction::Transaction;
use auctions_pub_ledger::auction_app::auction_operation::client::create_user;
use auctions_pub_ledger::auction_app::auction_operation::client::get_auction_house;
use auctions_pub_ledger::auction_app::auction_operation::client::get_user;
use auctions_pub_ledger::auction_app::auction_operation::client::send_transaction;
use auctions_pub_ledger::auction_app::notifications::notify_server::notification_server;
use auctions_pub_ledger::auction_app::user::UserActivity;
use auctions_pub_ledger::auction_app::user::{
    add_credits, load_users_from_file, register_user, User,
};
use auctions_pub_ledger::cryptography::ecdsa_keys::load_ecdsa_keys;
use chrono::{Duration, Utc};
use colored::*;
use k256::ecdsa::SigningKey;
use k256::ecdsa::{signature::Signer, Signature};
use local_ip_address::local_ip;
use sha256::digest;
use std::env;
use std::io::{self, Write};
use tokio::task;

#[cfg(not(target_os = "windows"))]
fn clear_screen() {
    std::process::Command::new("clear").status().unwrap();
}
const BOOTSTRAP_NODE_ADDRES: &str = "10.10.0.2";

#[tokio::main]
async fn main() {
    task::spawn(notification_server());
    clear_screen();
    let args: Vec<String> = env::args().collect();
    println!("Welcome to the BidBuddie's Auction System!");

    println!("Please select an option:\n1. Login\n2. Register");
    let mut option = String::new();
    io::stdin()
        .read_line(&mut option)
        .expect("Failed to read line");

    let mut user = match option.trim() {
        "1" => login().await,
        "2" => register_user(BOOTSTRAP_NODE_ADDRES).await,
        _ => {
            println!("Invalid option, please try again.");
            return;
        }
    };
    pause();

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

async fn login() -> User {
    println!("Please enter your Username:");
    let mut username = String::new();
    io::stdin()
        .read_line(&mut username)
        .expect("Failed to read line");
    let username = username.trim();

    let user: User = load_users_from_file(&format!("users/{}.json", &username))
        .await
        .expect("Login error please try again...");

    println!(
        "{}",
        format!("Login successful\nWelcome back {}", user.user_name).green()
    );

    user
}

async fn auctions_menu(user: &mut User, peers_list: Vec<String>) {
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
            "1" => join_auction(user, &peers_list, private_key.clone()).await,
            "2" => create_auction(user, &peers_list, private_key.clone()).await,
            "3" => current_auctions(&peers_list).await,
            "4" => history(user).await,
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
            "1" => view_profile(user).await,
            "2" => add_credits(BOOTSTRAP_NODE_ADDRES, user).await,
            "3" => history(user).await,
            "4" => break,
            _ => {
                println!("Invalid option, please try again.");
            }
        }
    }
}

async fn view_profile(user: &User) {
    let user = get_user(BOOTSTRAP_NODE_ADDRES, &user.uid).await.unwrap();

    clear_screen();
    println!("User Profile:");
    println!("Uid: {}", user.uid);
    println!("Username: {}", user.user_name);
    println!("Credits: ${}", user.credits);

    pause();
}

async fn join_auction(user: &mut User, dest_ip: &Vec<String>, private_key: SigningKey) {
    let mut user = get_user(BOOTSTRAP_NODE_ADDRES, &user.uid).await.unwrap();

    clear_screen();

    get_auction_house(dest_ip)
        .await
        .expect("error geting acution from peers");

    let _ = list_auctions().await;
    let auction_signature;
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
        match trimmed_input.parse::<String>() {
            Ok(value) => {
                auction_signature = value;
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

    let signed_content =
        digest(auction_signature.clone() + &user.uid.clone() + &amount.to_string());
    let signature: Signature = private_key.sign(signed_content.clone().as_bytes());
    if user.credits >= amount {
        let bid = Bid {
            bidder: user.uid.clone(),
            amount,
            signature: hex::encode(signature.clone().to_bytes()),
            auction_signature: auction_signature,
        };
        let local_ip_address = local_ip().unwrap();

        match send_transaction(
            Transaction::Bid(bid.clone()),
            &dest_ip[0],
            local_ip_address.to_string(),
        )
        .await
        {
            Ok(_result) => {
                println!(
                    "Transaction successfully created for auction:  {:?} ",
                    bid.clone().signature
                );
                let activity: UserActivity = UserActivity {
                    activity_type: "Bid".to_string(),
                    auction_signature: bid.auction_signature,
                    amount: bid.amount,
                };
                user.activity.push(activity);
                let user_str = serde_json::to_string(&user).unwrap();
                let _ = create_user(BOOTSTRAP_NODE_ADDRES, &user_str).await;
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

async fn create_auction(user: &mut User, dest_ip: &Vec<String>, private_key: SigningKey) {
    let mut user = get_user(BOOTSTRAP_NODE_ADDRES, &user.uid).await.unwrap();

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
        .expect("Please enter a valid number of minutes");
    let end_time = start_time + Duration::minutes(duration);

    get_auction_house(dest_ip)
        .await
        .expect("error geting acution from peers");

    list_auctions().await;

    let signed_content =
        digest(item_name.trim().to_string() + &starting_bid.to_string() + &user.uid.clone());

    let signature: Signature = private_key.sign(signed_content.as_bytes());

    // Use user.uid to pass the creator's uid to the new auction
    let auction = Auction::new(
        item_name.trim().to_string(),
        start_time,
        end_time,
        starting_bid,
        user.uid.clone(), // Pass the user's uid as the creator
        hex::encode(signature.to_bytes()),
        vec![],
    );

    let local_ip_address = local_ip().unwrap();
    match send_transaction(
        Transaction::Auction(auction.clone()),
        &dest_ip[0],
        local_ip_address.to_string(),
    )
    .await
    {
        Ok(_result) => {
            println!("Auction created successfully:  {:?}", auction.signature);
            let activity: UserActivity = UserActivity {
                activity_type: "AuctionCreation".to_string(),
                auction_signature: auction.signature,
                amount: auction.starting_bid,
            };
            user.activity.push(activity);
            let user_str = serde_json::to_string(&user).unwrap();
            let _ = create_user(BOOTSTRAP_NODE_ADDRES, &user_str).await;
        }
        Err(e) => {
            println!("error {}", e);
        }
    }

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

async fn history(user: &User) {
    let user: User = get_user(BOOTSTRAP_NODE_ADDRES, &user.uid).await.unwrap();

    clear_screen();
    println!("Auction Activity:");
    println!(
        "|{:<130} | {:<15} | {:<13} | {:<10}|",
        "ID", "Activity type", "bidding price", "Auction Winner"
    );
    for activity in user.activity.iter() {
        let winner_checker = user.auctions_winner.contains(&activity.auction_signature);

        println!(
            "|{:<130} | {:<15} | {:<13} | {:<10}|",
            activity.auction_signature, activity.activity_type, activity.amount, winner_checker
        );
    }
    //user.list_participated_auctions();

    pause();
}

fn pause() {
    let mut pause = String::new();
    println!("\nPress Enter to continue...");
    io::stdin().read_line(&mut pause).unwrap();
}
