use crate::auction_app::auction::AuctionHouse;
use crate::auction_app::auction_operation::client::update_user;
use crate::auction_server::blockchain::block_generator;
use crate::auction_server::blockchain::Blockchain;
use crate::auction_server::blockchain_operator::block_peer_validator_client;
use crate::auction_server::blockchain_pos::{pos_miner_puzzle, puzzle_builder};
use crate::auction_server::blockchain_pow::{block_handler, blockchain_handler};
use crate::kademlia_node_search::node::Node;
use crate::kademlia_node_search::node_functions::routing_table::Bucket;
use chrono::Utc;
use colored::*;
use std::sync::Arc;
use std::vec::Vec;
use tokio::sync::Mutex;
// Function executed in loop to validate if auctions is closed (10 in 10 seconds)
// if aution is closed create transaction and prepare to generate block
pub async fn auctions_validator(
    dest_ip: Arc<Mutex<Node>>,
    shared_blockchain_vector: Arc<Mutex<Vec<Blockchain>>>,
    shared_auction_house: Arc<Mutex<AuctionHouse>>,
    validation_type: Option<String>,
) {
    let mut byte_count = 0;
    let mut tx: Vec<String> = Vec::new();

    loop {
        let mut auction_house = shared_auction_house.lock().await;
        let shared_node = dest_ip.lock().await;
        let rt = <Vec<Bucket> as Clone>::clone(&shared_node.routing_table.lock().await.buckets)
            .into_iter()
            .map(|x| {
                x.nodes
                    .into_iter()
                    .map(|node_info| node_info.addr.to_string())
                    .collect::<Vec<String>>()
            })
            .flatten()
            .collect::<Vec<String>>();

        for auction in auction_house.auctions.iter_mut() {
            if &auction.end_time < &Utc::now()
                && !tx.contains(&auction.signature)
                && !auction.bids.is_empty()
                && auction.active
            {
                // update winning user
                let _ = update_user("10.10.0.2", &auction.bids.last().unwrap()).await;

                // insert into transaction vector
                tx.push(auction.signature.to_string());

                auction.active = false;
                println!(
                    "{}",
                    format!("Auction expired: {:?}\n", auction.signature).blue()
                );
                byte_count += &auction.signature.as_bytes().len();
                if byte_count >= 5 {
                    let mut result_validation = false;
                    let new_block = block_generator(shared_blockchain_vector.clone(), tx).await;

                    let mut new_block_to_validate = new_block.clone();
                    new_block_to_validate.hash = "".to_string();
                    new_block_to_validate.nounce = 0;

                    let mut list_peer_validation: Vec<bool> = Vec::new();

                    let validation_nounce =
                        block_handler(&mut shared_blockchain_vector.clone(), new_block.clone())
                            .await;
                    if validation_nounce == new_block.nounce {
                        result_validation = true;
                    }

                    match validation_type.as_ref() {
                        Some(s) if s == "pos" => {
                            // create puzzle with solution
                            let mut peer_puzzle_winner = "".to_string();
                            let (puzzle_set, puzzle_solution_set) = puzzle_builder().await;
                            let mut handle_puzzle_results = Vec::new();

                            for peer in rt.clone() {
                                // send puzzle to peers
                                let handle_puzzle_result = tokio::task::spawn(pos_miner_puzzle(
                                    puzzle_set.clone(),
                                    peer.clone().split(':').next().unwrap().to_owned(),
                                ));
                                handle_puzzle_results.push(handle_puzzle_result);
                            }

                            for handle_puzzle_result in handle_puzzle_results {
                                match handle_puzzle_result.await {
                                    Ok(Ok((solution_result, peer_ip))) => {
                                        if solution_result == puzzle_solution_set {
                                            peer_puzzle_winner = peer_ip;
                                            break;
                                        }
                                    }
                                    Ok(Err(e)) => eprintln!("Puzzle result error: {}", e),
                                    Err(e) => eprintln!("Puzzle request error: {}", e),
                                }
                            }
                            if !peer_puzzle_winner.is_empty() {
                                println!(
                                    "{}",
                                    format!("Node Puzzle Winner: {}\n", peer_puzzle_winner).green()
                                );
                                // send block to the first node that retrieves the puzzle corretly
                                let block_validated_nounce = block_peer_validator_client(
                                    new_block_to_validate,
                                    peer_puzzle_winner.clone(),
                                )
                                .await
                                .expect("error getting validation from peer");

                                if block_validated_nounce == new_block.clone().nounce {
                                    list_peer_validation.push(true);
                                    println!(
                                        "{}",
                                        format!(
                                            "Peer validation {}: {}\n",
                                            &peer_puzzle_winner, true
                                        )
                                        .green()
                                    );
                                }
                            }
                        }
                        Some(s) if s == "pow" => {
                            let mut handle_peer_validation_results = Vec::new();

                            for peer in rt.clone() {
                                // send puzzle to peers
                                let handle_puzzle_result =
                                    tokio::task::spawn(block_peer_validator_client(
                                        new_block_to_validate.clone(),
                                        peer.clone().split(':').next().unwrap().to_owned(),
                                    ));
                                handle_peer_validation_results.push(handle_puzzle_result);
                            }

                            for handle_peer_validation_result in handle_peer_validation_results {
                                match handle_peer_validation_result.await {
                                    Ok(Ok(block_validated_nounce)) => {
                                        if block_validated_nounce == new_block.clone().nounce {
                                            list_peer_validation.push(true);
                                        }
                                    }

                                    Ok(Err(e)) => eprintln!("Peer validation result error: {}", e),
                                    Err(e) => eprintln!("Peer validation request error: {}", e),
                                }
                            }
                        }
                        _ => {}
                    }

                    tx = Vec::new();
                    if result_validation
                        && list_peer_validation
                            .iter()
                            .any(|r_validation| r_validation == &true)
                    {
                        blockchain_handler(&mut shared_blockchain_vector.clone()).await;
                    }

                    byte_count = 0;
                }
            }
        }
    }
}
