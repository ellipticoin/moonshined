use crate::{
    bridge_abi::BRIDGE_ABI,
    constants::{ADDRESS, CHAIN_ID, PROVIDER},
};
use crate::{rlp, BRIDGE_ADDRESS};
use ellipticoin_contracts::bridge::PendingWithdrawl;
use ellipticoin_peerchain_ethereum::crypto::sign;
use ethereum_abi::Value;
use num_bigint::BigUint;
use num_traits::FromPrimitive;
use std::collections::HashMap;

pub async fn process_withdrawl(pending_withdrawl: &PendingWithdrawl) {
    let transaction_number = PROVIDER.get_transaction_count(&ADDRESS).await.unwrap();
    let transaction_data = vec![
        BRIDGE_ABI.functions[6].method_id().to_vec(),
        Value::encode(&vec![
            Value::Address(pending_withdrawl.token.0.into()),
            Value::Address(pending_withdrawl.to.0.into()),
            Value::Uint(pending_withdrawl.amount.into(), 64),
            Value::Uint(pending_withdrawl.id.into(), 64),
        ]),
    ]
    .concat();

    let mut transaction = vec![
        encode_u64(transaction_number),
        encode_u64(get_gas_price("fast").await),
        encode_u64(1000000u64),
        BRIDGE_ADDRESS.0.to_vec(),
        vec![],
        transaction_data,
        encode_u64(CHAIN_ID),
        vec![],
        vec![],
    ];
    let signature = sign(&rlp::encode(transaction.clone()));
    transaction[6] = encode_u64(CHAIN_ID * 2 + 35 + signature.v[0] as u64);
    transaction[7] = signature.r.to_vec();
    transaction[8] = signature.s.to_vec();
    PROVIDER
        .send_raw_transaction(&rlp::encode(transaction))
        .await
        .unwrap();
}

pub async fn get_gas_price(priority: &str) -> u64 {
    let res = surf::get("https://gasstation-mainnet.matic.network/")
        .await
        .unwrap()
        .body_json::<HashMap<String, f64>>()
        .await
        .unwrap();
    (res.get(priority).unwrap() * 1000000000.0) as u64
}

pub fn encode_u64(n: u64) -> Vec<u8> {
    let bytes = BigUint::from_u64(n).unwrap().to_bytes_be().to_vec();
    if let Some(first_non_zero_byte) = bytes.iter().position(|b| *b != 0u8) {
        bytes[first_non_zero_byte..bytes.len()].to_vec()
    } else {
        vec![]
    }
}
