mod kademlia {
    tonic::include_proto!("kademlia"); // The string here should match the package name in your .proto file
}

mod node;

use node::Node;
use tonic::transport::Server;
use kademlia::kademlia_server::KademliaServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let node = Node {};

    println!("Kademlia node listening on {}", addr);
    
    Server::builder()
        .add_service(KademliaServer::new(node))
        .serve(addr)
        .await?;

    Ok(())
}
