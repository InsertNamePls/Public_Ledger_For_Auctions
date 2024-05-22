use crate::auction_server::blockchain::{Block, Blockchain};
use crate::auction_server::blockchain_pos::{Puzzle, PuzzleSet};
use crate::auction_server::blockchain_pow::block_handler;
use crate::blockchain_grpc::blockchain_grpc_client::BlockchainGrpcClient;
use crate::blockchain_grpc::blockchain_grpc_server::BlockchainGrpc;
use crate::blockchain_grpc::ProofOfStakePuzzleRequest;
use crate::blockchain_grpc::ProofOfStakePuzzleResponse;
use crate::blockchain_grpc::{
    ProofOfWorkRequest, ProofOfWorkResponse, RetrieveBlockchainRequest, RetrieveBlockchainResponse,
};
use std::sync::Arc;
use std::vec::Vec;
use tokio::sync::Mutex;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use tonic::{Request, Response, Status};

use sha256::digest;
const DIFICULTY: usize = 2;
#[derive(Default, Debug, Clone)]
pub struct BlockchainServer {
    pub shared_blockchain_state: Arc<Mutex<Vec<Blockchain>>>,
}

type BlockchainGrpcResult<T> = Result<Response<T>, Status>;
#[tonic::async_trait]
impl BlockchainGrpc for BlockchainServer {
    async fn retrieve_blockchain(
        &self,
        _request: Request<RetrieveBlockchainRequest>,
    ) -> BlockchainGrpcResult<RetrieveBlockchainResponse> {
        // here get the shared state and lock it
        let bch = &self.shared_blockchain_state.lock().await;
        // send main blockchain to client!!!
        let blockchain = serde_json::to_string(&bch.get(0).unwrap()).unwrap();

        Ok(Response::new(RetrieveBlockchainResponse { blockchain }))
    }
    async fn proof_of_work(
        &self,
        request: Request<ProofOfWorkRequest>,
    ) -> BlockchainGrpcResult<ProofOfWorkResponse> {
        let block: Block = serde_json::from_str(&request.into_inner().block).unwrap();

        println!("incoming block from peer: {:?}", block.clone());
        let validation =
            block_handler(&mut self.shared_blockchain_state.clone(), block.clone()).await;

        Ok(Response::new(ProofOfWorkResponse { validation }))
    }
    async fn proof_of_stake_puzzle(
        &self,
        request: Request<ProofOfStakePuzzleRequest>,
    ) -> BlockchainGrpcResult<ProofOfStakePuzzleResponse> {
        let puzzleset: PuzzleSet = serde_json::from_str(&request.into_inner().puzzle).unwrap();
        let mut puzzle_solution: PuzzleSet = PuzzleSet {
            puzzle_list: vec![],
        };
        for puzzle in puzzleset.puzzle_list.iter() {
            let mut nounce_solution = 0;

            let mut solution_code =
                digest(puzzle.solution.to_owned() + &nounce_solution.to_string());

            loop {
                if puzzle.to_owned().code != solution_code {
                    nounce_solution += 1;

                    solution_code = digest(puzzle.solution.clone() + &nounce_solution.to_string());
                } else {
                    puzzle_solution.puzzle_list.push(Puzzle {
                        solution: puzzle.solution.clone(),
                        nounce: nounce_solution,
                        code: solution_code,
                    });
                    break;
                }
            }
            println!(
                "puzzle solved for code {}: {}",
                puzzle.code, nounce_solution
            );
        }
        println!("Proof of Stake puzzle solution: {:?}", puzzle_solution);
        let result = serde_json::to_string(&puzzle_solution).unwrap();
        Ok(Response::new(ProofOfStakePuzzleResponse { result }))
    }
}

// blockchain client
pub async fn blockchain_client(
    peer: String,
) -> Result<BlockchainGrpcClient<Channel>, Box<dyn std::error::Error>> {
    let ca = std::fs::read_to_string("tls/rootCA.crt")?;
    let tls = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(ca))
        .domain_name("auctiondht.com");
    let channel = Channel::builder(format!("https://{}:3001", peer).parse().unwrap())
        .tls_config(tls)
        .unwrap()
        .connect()
        .await
        .unwrap();
    let client = BlockchainGrpcClient::new(channel);
    Ok(client)
}

pub async fn blockchain_client_async(
    peer: String,
) -> Result<BlockchainGrpcClient<Channel>, Box<dyn std::error::Error + Send + Sync>> {
    let ca = std::fs::read_to_string("tls/rootCA.crt")?;
    let tls = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(ca))
        .domain_name("auctiondht.com");
    let channel = Channel::builder(format!("https://{}:3001", peer).parse().unwrap())
        .tls_config(tls)
        .unwrap()
        .connect()
        .await
        .unwrap();

    let client = BlockchainGrpcClient::new(channel);

    Ok(client)
}
