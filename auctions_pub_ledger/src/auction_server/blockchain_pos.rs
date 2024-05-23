use crate::auction_server::blockchain::Blockchain;
use crate::auction_server::blockchain_operation::client::blockchain_client_async;
use crate::blockchain_grpc::ProofOfStakePuzzleRequest;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha256::digest;
use std::vec::Vec;
use std::{fs, usize};
use tokio::sync::watch::error;
use tonic::transport::Error;
const DIFICULTY: usize = 2;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Puzzle {
    pub solution: String,
    pub nounce: u64,
    pub code: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PuzzleSet {
    pub puzzle_list: Vec<Puzzle>,
}
impl Puzzle {
    pub fn new() -> Self {
        Puzzle {
            solution: String::new(),
            nounce: 0,
            code: String::new(),
        }
    }
    pub fn generate_puzzle(puzzle: &mut Puzzle) {
        let puzzle_str: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(20)
            .map(char::from)
            .collect();
        println!("{}", puzzle_str.clone());
        let target = "0".repeat(DIFICULTY);
        loop {
            if !puzzle.code.starts_with(&target) {
                puzzle.nounce += 1;
                puzzle.code = digest(puzzle_str.clone() + &puzzle.nounce.to_string());
            } else {
                break;
            }
        }

        puzzle.solution = puzzle_str;
    }
}

pub async fn puzzle_builder() -> (PuzzleSet, PuzzleSet) {
    let mut puzzle_set = PuzzleSet {
        puzzle_list: vec![],
    };
    let mut puzzle_solution_set = PuzzleSet {
        puzzle_list: vec![],
    };

    for i in 0..5 {
        print!("Building PoS puzzle: {}", i);
        let mut puzzle = Puzzle::new();
        Puzzle::generate_puzzle(&mut puzzle);
        puzzle_solution_set.puzzle_list.push(puzzle.clone());

        puzzle.nounce = 0;
        puzzle_set.puzzle_list.push(puzzle);
    }
    (puzzle_set, puzzle_solution_set)
}

pub async fn pos_miner_puzzle(
    puzzleset: PuzzleSet,
    peer: String,
) -> Result<(PuzzleSet, String), Box<dyn std::error::Error + Send + Sync>> {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let mut client = blockchain_client_async(peer.clone()).await?;

    let puzzle = serde_json::to_string(&puzzleset).unwrap();
    let request = tonic::Request::new(ProofOfStakePuzzleRequest { puzzle });
    let response = client.proof_of_stake_puzzle(request).await?;

    let result_response = response.into_inner().result;
    let solution_puzzle: PuzzleSet = serde_json::from_str(&result_response).unwrap();

    Ok((solution_puzzle, peer))
}

pub async fn save_blockchain_locally(blockchain: &Blockchain, file_path: &str) {
    let chain_serialized = serde_json::to_string_pretty(&blockchain).unwrap();
    fs::write(file_path, chain_serialized).expect("Unable to write file");
}
