use k256::ecdsa::{recoverable, signature::Signer};

use crate::{constants::PRIVATE_KEY, signature::Signature};

use sha2::{Digest, Sha256};
use sha3::Keccak256;
use std::convert::TryInto;

pub fn sign(message: &[u8]) -> Signature {
    let signature: recoverable::Signature = PRIVATE_KEY.sign(&message);

    let signature_bytes = k256::ecdsa::signature::Signature::as_bytes(&signature);
    Signature {
        r: signature_bytes[0..32].try_into().unwrap(),
        s: signature_bytes[32..64].try_into().unwrap(),
        v: [signature_bytes[64]],
    }
}
pub fn keccak256(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Keccak256::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}

pub fn sha256(message: Vec<u8>) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(message);
    hasher.finalize().into()
}
