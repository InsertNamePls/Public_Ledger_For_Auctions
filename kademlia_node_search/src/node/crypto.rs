use ring::{rand as ring_rand, signature};

pub struct Crypto {}

impl Crypto {
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

    pub fn validate_message(message: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
        let peer_public_key = signature::UnparsedPublicKey::new(&signature::ED25519, public_key);
        peer_public_key.verify(message, signature).is_ok()
    }
}
