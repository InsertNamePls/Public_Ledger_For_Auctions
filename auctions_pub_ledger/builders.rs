fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/auctions.proto")?;
    tonic_build::compile_protos("proto/blockchain.proto")?;
    tonic_build::compile_protos("proto/kademlia.proto")?;
    tonic_build::compile_protos("proto/notification.proto")?;
    Ok(())
}
