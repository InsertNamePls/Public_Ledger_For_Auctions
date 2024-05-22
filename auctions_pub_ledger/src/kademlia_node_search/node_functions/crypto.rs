use ring::{rand as ring_rand, signature};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::kademlia_node_search::config::REPLAY_WINDOW;

pub struct Crypto {}

impl Crypto {
    pub fn validate_request(
        timestamp: i64,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> bool {
        return Crypto::validate_message_timestamp(timestamp)
            && Crypto::validate_message_authenticity(message, signature, public_key);
    }

    pub fn create_keypair() -> Result<signature::Ed25519KeyPair, &'static str> {
        let rng = ring_rand::SystemRandom::new();

        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|_| "Failed to generate pkcs8 bytes")?;
        let keypair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .map_err(|_| "Failed to create keypair from pkcs8 bytes")?;

        Ok(keypair)
    }

    pub fn sign_message(keypair: &signature::Ed25519KeyPair, message: &[u8]) -> Vec<u8> {
        keypair.sign(message).as_ref().to_vec()
    }

    pub fn validate_message_timestamp(timestamp: i64) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        // Ensure the timestamp is within REPLAY_WINDOW of the current time to prevent replay attacks
        return current_time - timestamp.abs() <= REPLAY_WINDOW;
    }

    pub fn validate_message_authenticity(
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> bool {
        let peer_public_key = signature::UnparsedPublicKey::new(&signature::ED25519, public_key);
        peer_public_key.verify(message, signature).is_ok()
    }
}
