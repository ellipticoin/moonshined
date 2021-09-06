use super::{
    errors,
    errors::{Result, PARSE_ERROR},
};
use crate::{aquire_db_read_lock, constants::DB, transaction::SignedTransaction};
use ellipticoin_contracts::{system::Transaction, System};
use ellipticoin_peerchain_ethereum::signature::Signature;
use ellipticoin_peerchain_ethereum::{abi::decode_action, rlp};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use serde_json::Value;

pub async fn parse_block_tag(value: &Value) -> Result<u64> {
    if value == "latest" {
        let mut db = aquire_db_read_lock!();
        Ok(System::get_block_number(&mut db))
    } else {
        parse_u64(value)
    }
}

pub fn parse_u64(value: &Value) -> Result<u64> {
    BigUint::from_bytes_be(&parse_bytes(value)?)
        .to_u64()
        .ok_or(PARSE_ERROR)
}

pub fn parse_bytes(value: &Value) -> Result<Vec<u8>> {
    Ok(hex::decode(value.as_str().unwrap_or("").trim_start_matches("0x")).or(Err(PARSE_ERROR))?)
}

pub fn parse_signed_transaction(value: &Value) -> Result<SignedTransaction> {
    let transaction_attributes = rlp::decode(&parse_bytes(&value)?);
    Ok(SignedTransaction(
        Transaction {
            action: decode_action(
                &transaction_attributes[3],
                &transaction_attributes[4],
                &transaction_attributes[5],
            )
            .map_err(|_| errors::PARSE_ERROR)?,
            transaction_number: BigUint::from_bytes_le(&transaction_attributes[0])
                .to_u64()
                .ok_or(errors::PARSE_ERROR)?,
        },
        Signature::from_v_r_s(
            &transaction_attributes[6],
            &transaction_attributes[7],
            &transaction_attributes[8],
        )
        .map_err(|_| errors::PARSE_ERROR)?,
    ))
}
