use auctions_pub_ledger::auction_app::auction::AuctionHouse;
use auctions_pub_ledger::auction_app::auction_operation::server::auction_server;
use auctions_pub_ledger::auction_server::auction_validator::auctions_validator;
use auctions_pub_ledger::auction_server::blockchain::init_blockchain;
use auctions_pub_ledger::auction_server::blockchain::Blockchain;
use auctions_pub_ledger::auction_server::blockchain_operation::server::blockchain_server;
use auctions_pub_ledger::auction_server::blockchain_operator::get_remote_blockchain;
use auctions_pub_ledger::auction_server::blockchain_operator::save_blockchain_locally;
use clap::{Arg, Command};
use tokio::task;

use auctions_pub_ledger::kademlia_node_search::node::run_server;
use auctions_pub_ledger::kademlia_node_search::node::Node;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use std::vec::Vec;
use tokio::sync::Mutex;

async fn destributed_auction_operator(
    blockchain_vector: Vec<Blockchain>,
    bootstrap_addr: Option<&String>,
    addr: SocketAddr,
    mining_type: Option<&String>,
) {
    let shared_blockchain_vector = Arc::new(Mutex::new(blockchain_vector));
    println!("{:?}", bootstrap_addr.cloned());
    let kademlia_node: Arc<Mutex<Node>> = Node::new(addr, bootstrap_addr.cloned()).await.unwrap();
    // initialize auction house by importing from file
    let data = fs::read_to_string("auction_data.json").expect("Unable to read file");
    let auction_house: AuctionHouse =
        serde_json::from_str(&data).expect("Failed to deserialize JSON");

    let share_auction_house = Arc::new(Mutex::new(auction_house));

    let task1 = task::spawn(auction_server(
        share_auction_house.clone(),
        kademlia_node.clone(),
    ));
    let task2 = task::spawn(auctions_validator(
        kademlia_node.clone(),
        shared_blockchain_vector.clone(),
        share_auction_house.clone(),
        mining_type.cloned(),
    ));
    let task3 = task::spawn(blockchain_server(shared_blockchain_vector.clone()));
    let task4 = task::spawn(run_server(addr, kademlia_node.clone()));
    //let task5 = task::spawn(loop_func(kademlia_node.clone()));
    task1.await.unwrap();
    task2.await.unwrap();
    task3.await.unwrap();
    task4.await.unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Destributed Blockchain with S/Kademlia DHT")
        .version("1.0")
        .subcommand(
            Command::new("init_blockchain")
                .about("Runs in server mode")
                .arg(
                    Arg::new("mining_type")
                        .help("type of mining options:(PoW or PoS)")
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .default_value("PoS")
                        .index(1),
                )
                .arg(
                    Arg::new("bootstrap")
                        .help("Optional address of the bootstrap node")
                        .value_parser(clap::value_parser!(String))
                        .required(false)
                        .index(2),
                ),
        )
        .subcommand(
            Command::new("join_blockchain")
                .about("Runs in client mode")
                .arg(
                    Arg::new("mining_type")
                        .help("type of mining options:(pow or pos)")
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .default_value("pow")
                        .index(1),
                )
                .arg(
                    Arg::new("bootstrap")
                        .help("Optional address of the bootstrap node")
                        .value_parser(clap::value_parser!(String))
                        .required(false)
                        .index(2),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("init_blockchain", server_matches)) => {
            let mut blockchain_vector: Vec<Blockchain> = Vec::new();
            println!("init blockchain with genesis block");

            let bchain = init_blockchain().await;

            blockchain_vector.push(bchain.clone());
            let bootstrap_addr = server_matches.get_one::<String>("bootstrap");
            let mining_type = server_matches.get_one::<String>("mining_type");
            let addr = env::var("KADEMLIA_LOCAL_NODE").expect("MY_ENV_VAR is not set");

            let addr = addr.parse::<SocketAddr>().unwrap();

            save_blockchain_locally(&bchain, "blockchain_active/blockchain_0.json").await;
            destributed_auction_operator(blockchain_vector, bootstrap_addr, addr, mining_type)
                .await;
        }
        Some(("join_blockchain", server_matches)) => {
            let bootstrap_addr = server_matches.get_one::<String>("bootstrap");
            let mining_type = server_matches.get_one::<String>("mining_type");
            let addr = env::var("KADEMLIA_LOCAL_NODE").expect("MY_ENV_VAR is not set");

            let addr = addr.parse::<SocketAddr>().unwrap();
            match get_remote_blockchain(
                bootstrap_addr
                    .cloned()
                    .unwrap()
                    .split(':')
                    .next()
                    .unwrap()
                    .to_owned(),
            )
            .await
            {
                Ok(result) => {
                    println!("{:?}", result);
                    destributed_auction_operator(result, bootstrap_addr, addr, mining_type).await;
                }
                Err(e) => {
                    println!("error {}", e);
                }
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
