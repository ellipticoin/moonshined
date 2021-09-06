use super::helpers::left_pad;
use num_bigint::BigUint;
use num_traits::{pow, Zero};
use serde_json::{json, Value};

pub fn encode_token_amount(amount: u64) -> Value {
    encode_amount(BigUint::from(amount) * BigUint::from(pow(BigUint::from(10u32), 12)))
}

pub fn encode_amount(amount: BigUint) -> Value {
    if amount == Zero::zero() {
        json!("0x0")
    } else {
        json!(format!(
            "0x{}",
            hex::encode(amount.to_bytes_be()).trim_start_matches('0')
        ))
    }
}

pub fn encode_u64_as_hash(n: u64) -> Value {
    encode_bytes(&left_pad(&n.to_be_bytes(), 32))
}

pub fn encode_bytes(bytes: &[u8]) -> Value {
    json!(format!("0x{}", hex::encode(bytes)))
}
