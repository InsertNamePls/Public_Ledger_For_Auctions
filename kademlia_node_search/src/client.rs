use rand::RngCore;
use bytes::Bytes;
use crate::node::crypto::Crypto;
use crate::node::client::Client;

pub async fn run_client(target: &str, command: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Generate keypair
    let keypair = Crypto::create_keypair()?;
    let client = Client::new();
    let id = generate_bytes(20).to_vec();

    match command {
        "store" => {
            // Collect user input for the key and value
            println!("Enter a key (text) for the store operation:");
            let mut key_input = String::new();
            std::io::stdin().read_line(&mut key_input)?;
            let key_bytes = key_input.trim().to_string().into_bytes();
            let key = Bytes::from(key_bytes);

            println!("Enter a value (text) for the store operation:");
            let mut value_input = String::new();
            std::io::stdin().read_line(&mut value_input)?;
            let value_bytes = value_input.trim().to_string().into_bytes();
            let value = Bytes::from(value_bytes);

            let store_request = client.create_store_node_request(&keypair,id, key.to_vec(), value.to_vec());
            let response = client.send_store_request(store_request, target.to_string()).await?;

            println!("Store response: {:?}", response);
        },
        "find_value" => {
            // Collect user input for the key
            println!("Enter a key (text) for the find_value operation:");
            let mut key_input = String::new();
            std::io::stdin().read_line(&mut key_input)?;
            let key_bytes = key_input.trim().to_string().into_bytes();
            let key = Bytes::from(key_bytes);

            let find_value_request = client.create_find_value_request(&keypair,id, key.to_vec());
            let response = client.send_find_value_request(find_value_request, target.to_string()).await?;

            println!("Find value response: {:?}", response);
        },
        "find_node" => {
            // Collect user input for the target node ID
            println!("Enter a target node ID (hex) for the find_node operation:");
            let mut target_node_id_input = String::new();
            std::io::stdin().read_line(&mut target_node_id_input)?;
            let target_node_id_bytes = hex::decode(target_node_id_input.trim()).expect("Invalid hex input for target node ID");
            let target_node_id = Bytes::from(target_node_id_bytes);

            let find_node_request = client.create_find_node_request(
                &keypair,
                id,
                "127.0.0.1:10020".to_string(),
                target_node_id.to_vec()
            );

            let response = client.send_find_node_request(find_node_request, target.to_string()).await?;

            println!("Find node response: {:?}", response);
        },
        "ping" => {
            let ping_request = client.create_ping_request(&keypair,id, "127.0.0.1:10020".to_string());
            let response = client.send_ping_request(ping_request, target.to_string()).await?;

            println!("Ping response: {:?}", response);
        },
        _ => {
            println!("Unsupported command '{}'", command);
            return Ok(());
        }
    }

    Ok(())
}

/// Generates a random sequence of bytes of a given length
fn generate_bytes(len: usize) -> Bytes {
    let mut rng = rand::thread_rng();
    let mut id = vec![0u8; len]; // 160 bits
    rng.fill_bytes(&mut id);
    Bytes::from(id)
}
