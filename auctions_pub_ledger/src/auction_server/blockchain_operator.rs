use crate::blockchain::{validate_block, Block, Blockchain};
use chrono::Utc;
use std::sync::Arc;
use std::vec::Vec;
use std::{fs, usize};
use tokio::sync::Mutex;

const DIFICULTY: usize = 4;

pub async fn block_generator(
    shared_blockchain_vector: Arc<Mutex<Vec<Blockchain>>>,
    tx: Vec<String>,
) -> Block {
    let blockchain_vector = shared_blockchain_vector.lock().await;

    let main_blockchain = blockchain_vector.clone().get(0).unwrap().clone();

    let previous_block = main_blockchain.clone().blocks.last().unwrap().clone();
    println!("previous{:?}", previous_block);
    let mut block: Block = Block::new(
        previous_block.index + 1,
        previous_block.hash.clone(),
        0,
        Utc::now().timestamp_millis(),
        "".to_string(),
        tx,
    );

    block.mine_block(4);

    println!("generated_block -> {:?}\n", block);
    block
}

// based on local blockchains validate if block is valid
pub async fn validator(blockchain: Blockchain, block: Block) -> bool {
    if validate_block(&block, blockchain.blocks.last().unwrap(), DIFICULTY) {
        true
    } else {
        println!("block {} is invalid ", block.index);
        false
    }
}

pub async fn save_blockchain_locally(blockchain: &Blockchain, file_path: &str) {
    let chain_serialized = serde_json::to_string_pretty(&blockchain).unwrap();
    fs::write(file_path, chain_serialized).expect("Unable to write file");
}
use crate::blockchain_pow::block_handler;
use blockchain_grpc::blockchain_grpc_client::BlockchainGrpcClient;
use blockchain_grpc::blockchain_grpc_server::BlockchainGrpcServer;
use blockchain_grpc::{
    ProofOfWorkRequest, ProofOfWorkResponse, RetrieveBlockchainRequest, RetrieveBlockchainResponse,
};
use tonic::{transport::Server, Request, Response, Status};
#[derive(Default, Debug, Clone)]
pub struct BlockchainServer {
    shared_blockchain_state: Arc<Mutex<Vec<Blockchain>>>,
}

pub mod blockchain_grpc {
    tonic::include_proto!("blockchain_grpc");
}
type BlockchainGrpcResult<T> = Result<Response<T>, Status>;
use crate::blockchain_operator::blockchain_grpc::blockchain_grpc_server::BlockchainGrpc;
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
}

// blockchain Server
pub async fn blockchain_server(share_blockchain_vector: Arc<Mutex<Vec<Blockchain>>>) {
    let addr = "0.0.0.0:3001".parse().unwrap();

    Server::builder()
        .add_service(BlockchainGrpcServer::new(BlockchainServer {
            shared_blockchain_state: share_blockchain_vector,
        }))
        .serve(addr)
        .await;
}

// blockchain client
pub async fn get_remote_blockchain(
    peer: String,
) -> Result<Vec<Blockchain>, Box<dyn std::error::Error>> {
    let mut client = BlockchainGrpcClient::connect(format!("http://{}:3001", peer)).await?;
    let request = tonic::Request::new(RetrieveBlockchainRequest {});
    let response = client.retrieve_blockchain(request).await?;

    let mut blockchain_vector: Vec<Blockchain> = Vec::new();
    let bch_str = response.into_inner().blockchain;
    let main_blockchain: Blockchain = serde_json::from_str(&bch_str).unwrap();
    blockchain_vector.push(main_blockchain);

    Ok(blockchain_vector)
}
