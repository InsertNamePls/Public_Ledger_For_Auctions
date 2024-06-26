use crate::auction_server::blockchain::{Block, Blockchain};
use crate::auction_server::blockchain_operation::client::blockchain_client;
use crate::auction_server::blockchain_operation::client::blockchain_client_async;
use crate::blockchain_grpc::ProofOfWorkRequest;
use crate::blockchain_grpc::RetrieveBlockchainRequest;
use std::fs;
use std::vec::Vec;

pub async fn get_remote_blockchain(
    peer: String,
) -> Result<Vec<Blockchain>, Box<dyn std::error::Error>> {
    let mut client = blockchain_client(peer).await?;
    let request = tonic::Request::new(RetrieveBlockchainRequest {});
    let response = client.retrieve_blockchain(request).await?;

    let mut blockchain_vector: Vec<Blockchain> = Vec::new();
    let bch_str = response.into_inner().blockchain;
    let main_blockchain: Blockchain = serde_json::from_str(&bch_str).unwrap();
    blockchain_vector.push(main_blockchain);

    Ok(blockchain_vector)
}

pub async fn block_peer_validator_client(
    block_to_validate: Block,
    peer: String,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let mut client = blockchain_client_async(peer).await?;

    let block = serde_json::to_string(&block_to_validate).unwrap();
    let request = tonic::Request::new(ProofOfWorkRequest { block });
    let response = client.proof_of_work(request).await?;

    let nounce_str: String = response.into_inner().validation;

    Ok(nounce_str.parse::<u64>().unwrap())
}

pub async fn save_blockchain_locally(blockchain: &Blockchain, file_path: &str) {
    let chain_serialized = serde_json::to_string_pretty(&blockchain).unwrap();
    fs::write(file_path, chain_serialized).expect("Unable to write file");
}
