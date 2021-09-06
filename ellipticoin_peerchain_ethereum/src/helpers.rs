use crate::crypto::keccak256;
use ellipticoin_types::Address;
use k256::{ecdsa::VerifyingKey, elliptic_curve::sec1::ToEncodedPoint};
use std::convert::TryInto;
pub fn eth_address(verify_key: &VerifyingKey) -> Address {
    Address(
        keccak256(&verify_key.to_encoded_point(false).to_bytes()[1..])[12..]
            .try_into()
            .unwrap(),
    )
}

pub fn left_pad(s: &[u8], length: usize) -> Vec<u8> {
    let mut buf = vec![0u8; length - s.len()];
    buf.extend_from_slice(&s[0..s.len()]);
    buf
}
