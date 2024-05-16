use rand::RngCore;
use bytes::Bytes;
use crate::kademlia::kademlia_client::KademliaClient;

pub async fn run_client(target: &str, command: &str) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = format!("http://{}", target);
    let mut client = KademliaClient::connect(endpoint).await?;

    // Collect user input for the key
    println!("Enter a key (text) for the operation:");
    let mut user_input = String::new();
    std::io::stdin().read_line(&mut user_input)?;
    let user_input_bytes = user_input.trim().to_string().into_bytes();
    let key = Bytes::from(user_input_bytes);

    match command {
        "store" => {
            
        },
        "find_value" => {
           
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
