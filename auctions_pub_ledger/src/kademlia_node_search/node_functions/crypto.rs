use ring::{rand as ring_rand, signature};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::kademlia_node_search::config::{NONCE_INTERVAL, REPLAY_WINDOW};
#[derive(Debug)]
pub struct Crypto {
    receiver_nonces: Mutex<HashMap<Vec<u8>, i64>>, // Store the last nonce received from each ID
}

impl Crypto {
    pub fn new() -> Self {
        Crypto {
            receiver_nonces: Mutex::new(HashMap::new()),
        }
    }

    pub fn validate_request(
        &self,
        timestamp: i64,
        nonce: i64,
        id: &[u8],
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> bool {
        return self.validate_message_timestamp(timestamp)
            && self.validate_message_authenticity(message, signature, public_key)
            && self.validate_and_update_nonce(id, nonce);
    }

    pub fn create_keypair() -> Result<signature::Ed25519KeyPair, &'static str> {
        let rng = ring_rand::SystemRandom::new();

        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|_| "Failed to generate pkcs8 bytes")?;
        let keypair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .map_err(|_| "Failed to create keypair from pkcs8 bytes")?;

        Ok(keypair)
    }

    pub fn sign_message(&self, keypair: &signature::Ed25519KeyPair, message: &[u8]) -> Vec<u8> {
        keypair.sign(message).as_ref().to_vec()
    }

    pub fn validate_message_timestamp(&self, timestamp: i64) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        // Ensure the timestamp is within REPLAY_WINDOW of the current time to prevent replay attacks
        return current_time - timestamp.abs() <= REPLAY_WINDOW;
    }

    pub fn validate_message_authenticity(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> bool {
        let peer_public_key = signature::UnparsedPublicKey::new(&signature::ED25519, public_key);
        peer_public_key.verify(message, signature).is_ok()
    }

    fn validate_and_update_nonce(&self, id: &[u8], nonce: i64) -> bool {
        let mut receiver_nonces = self.receiver_nonces.lock().unwrap();
        let id_vec = id.to_vec();
        if let Some(&last_nonce) = receiver_nonces.get(&id_vec) {
            if nonce < last_nonce - NONCE_INTERVAL || nonce > last_nonce + NONCE_INTERVAL {
                return false;
            }
        }
        receiver_nonces.insert(id_vec, nonce);
        true
    }
}
