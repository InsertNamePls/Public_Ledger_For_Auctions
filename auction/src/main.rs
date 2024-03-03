use chrono::{NaiveDateTime};

mod transactions;
mod users;
use transactions::Auction;
use transactions::Transaction;
use users::*;


fn main(){
    let mut userx = User {
        name: "userx".to_string(),
        credits: 10.00
    };
    userx.new();
    let mut auction = Auction {
        transactions: vec![]
    };
    let transaction1 = Transaction {
        user: "UserA".to_string(),
        current_bidder: "".to_string(),
        bidding_price: 10.81,
        start_time:   NaiveDateTime::parse_from_str("2024-03-03 16:11:45", "%Y-%m-%d %H:%M:%S").unwrap(),
        end_time: NaiveDateTime::parse_from_str("2024-03-03 17:11:45", "%Y-%m-%d %H:%M:%S").unwrap(),
        active: true,
        subscription: true
    };
    let transaction2 = Transaction {
        user: "Userc".to_string(),
        current_bidder: "".to_string(),
        bidding_price: 220.1,
        start_time:   NaiveDateTime::parse_from_str("2024-03-03 16:11:45", "%Y-%m-%d %H:%M:%S").unwrap(),
        end_time: NaiveDateTime::parse_from_str("2024-03-03 17:11:45", "%Y-%m-%d %H:%M:%S").unwrap(),
        active: true,
        subscription: true
    };
    auction.new();
    auction.create_auction(transaction1);
    auction.create_auction(transaction2);
    auction.list_auctions();
    let current_credits = auction.create_bid(0, userx.name.clone(), userx.credits, 30.0);
    userx.credits = current_credits;
    auction.list_auctions();
    let current_credits = auction.create_bid(1, userx.name.clone(), userx.credits, 500.0);
    userx.credits = current_credits;
    auction.list_auctions();

    println!("{} {}", userx.name, userx.credits)
}
