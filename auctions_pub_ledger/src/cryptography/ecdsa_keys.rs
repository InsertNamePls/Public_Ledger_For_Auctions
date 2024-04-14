use elliptic_curve::generic_array::GenericArray;
use k256::ecdsa::SigningKey;
use k256::ecdsa::VerifyingKey;
use rand_core::OsRng;
use sha256::digest;
use std::fs;

use std::path::Path;

pub fn generate_ecdsa_keypair() -> (SigningKey, VerifyingKey) {
    let mut rng = OsRng;
    let signing_key = SigningKey::random(&mut rng);
    let verifying_key = VerifyingKey::from(&signing_key);

    fs::write(
        String::from(hex::encode(signing_key.as_ref().to_sec1_bytes())),
        signing_key.to_bytes(),
    );
    //fs::write("pubkey", verifying_key.to_sec1_bytes());
    //println!("{:?}", hex::encode(signing_key.to_bytes()));
    //println!("{:?}", hex::encode(signing_key.as_ref().to_sec1_bytes()));

    (signing_key, verifying_key)
}

pub fn load_ecdsa_keys(hash_public_key: String) -> (SigningKey, VerifyingKey) {
    let hex_private_key = fs::read(hash_public_key).unwrap();
    let signing_key =
        SigningKey::from_bytes(&GenericArray::clone_from_slice(&hex_private_key)).unwrap();

    let verifying_key: VerifyingKey = VerifyingKey::from(signing_key.clone());
    //println!("{:?}", hex::encode(signing_key.to_bytes()));
    //println!("{:?}", hex::encode(signing_key.as_ref().to_sec1_bytes()));
    (signing_key, verifying_key)
}

// pub fn ecdsa_keys() -> (SigningKey, VerifyingKey) {
//     if !Path::new("privkey").exists() {
//         // Generate a random signing key
//         println!("ECDSA Key does not exists. Creating...");
//         generate_ecdsa_keypair()
//     } else {
//         //load key pair
//
//         println!("ECDSA Key exists. Loading from file...");
//         load_ecdsa_keys()
//     }
// }
