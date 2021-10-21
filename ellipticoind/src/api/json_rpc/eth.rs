use super::{
    encoders::{encode_amount, encode_bytes, encode_token_amount, encode_u64_as_hash},
    errors::{Result, PARSE_ERROR},
    parsers::{parse_address, parse_block_tag, parse_bytes, parse_signed_transaction, parse_u64},
};
use crate::{
    aquire_db_read_lock,
    config::OPTS,
    constants::{DB, DEFAULT_GAS_LIMIT},
    transaction,
};
use ellipticoin_contracts::{
    token::tokens::{TOKEN_METADATA, USD},
    System,
};
use ellipticoin_peerchain_ethereum::abi::erc20_abi;
use ellipticoin_types::Address;
use num_bigint::BigUint;
use num_traits::Zero;
use serde_json::{json, Value};
use std::convert::TryFrom;
use std::convert::TryInto;

pub fn chain_id(_params: &Value) -> Result<Value> {
    Ok(json!(encode_amount(OPTS.chain_id.into())))
}

pub fn estimate_gas(_params: &Value) -> Result<Value> {
    Ok(json!(encode_amount(DEFAULT_GAS_LIMIT.into())))
}

pub async fn block_number(_params: &Value) -> Result<Value> {
    let mut db = aquire_db_read_lock!();
    Ok(encode_amount(
        (System::get_transaction_id_counter(&mut db)).into(),
    ))
}

pub async fn get_block_by_number(params: &Value) -> Result<Value> {
    let mut db = aquire_db_read_lock!();
    let block_number = parse_block_tag(&params[0]).await?;
    // if let Some(block) = System::get_blocks(&mut db).get(block_number as usize - 1usize) {
    if System::get_transaction_id_counter(&mut db) <= block_number {
        Ok(json!(
        {
              "hash":encode_u64_as_hash(block_number),
              "parentHash":encode_u64_as_hash(block_number - 1),
              "number": encode_amount(block_number.into()),
              "miner": encode_bytes(&[0; 20].to_vec()),
              "extraData": encode_bytes(&vec![]),
              "gasLimit": encode_amount(0u32.into()),
              "gasUsed": encode_amount(0u32.into()),
              "timestamp": encode_amount(0u32.into()),
              "transactions": vec![encode_u64_as_hash(block_number)],
          })
        .into())
    } else {
        Ok(json!(null))
    }
}

pub async fn get_block_by_hash(params: &Value) -> Result<Value> {
    get_block_by_number(&json!([
        encode_amount(BigUint::from_bytes_be(&parse_bytes(&params[0])?)),
        false
    ]))
    .await
}

pub async fn get_code(_params: &Value) -> Result<Value> {
    Ok(encode_bytes(&vec![]))
}

pub async fn get_transaction_count(params: &Value) -> Result<Value> {
    let address: Address = params[0]
        .as_str()
        .ok_or(PARSE_ERROR)?
        .try_into()
        .map_err(|_| PARSE_ERROR)?;
    let mut db = aquire_db_read_lock!();
    Ok(encode_amount(
        System::get_next_transaction_number(&mut db, address).into(),
    ))
}

pub async fn get_transaction_receipt(params: &Value) -> Result<Value> {
    let mut db = aquire_db_read_lock!();
    if System::get_transaction_id_counter(&mut db) >= parse_u64(&params[0])? {
        Ok(json!(
        {
          "blockHash":"0x0000000000000000000000000000000000000000000000000000000000000001",
          "blockNumber":"0x1",
          "cumulativeGasUsed": "0x0",
          "transactionIndex": "0x0",
          "effectiveGasPrice": "0x0",
          "transactionHash": params[0],
          "status":"0x1",
          "logs": [],
          "gasUsed":"0x0",
        }
          )
        .into())
    } else {
        Ok(json!(null))
    }
}

pub async fn gas_price(_params: &Value) -> Result<Value> {
    Ok(encode_token_amount(Zero::zero()))
}

pub async fn get_balance(params: &Value) -> Result<Value> {
    let mut db = aquire_db_read_lock!();
    let address: Address = params[0]
        .as_str()
        .ok_or(PARSE_ERROR)?
        .try_into()
        .map_err(|_| PARSE_ERROR)?;

    let balance =
        ellipticoin_contracts::Token::get_underlying_balance(&mut db, address.clone().into(), USD);

    Ok(encode_token_amount(balance))
}

pub async fn send_raw_transaction(params: &Value) -> Result<Value> {
    let signed_transaction = parse_signed_transaction(&params[0])?;
    let transaction_id = transaction::dispatch(signed_transaction).await.unwrap();
    //.map_err(|_| PARSE_ERROR)?;
    Ok(encode_u64_as_hash(transaction_id))
}

pub async fn call(params: &Value) -> Result<Value> {
    let to = parse_address(params[0].get("to").ok_or(PARSE_ERROR)?)?;
    if TOKEN_METADATA.get(&to).is_some() {
        let f = erc20_abi::ERC20_ABI
            .decode_input_from_slice(&parse_bytes(params[0].get("data").ok_or(PARSE_ERROR)?)?)
            .map_err(|_| PARSE_ERROR)?;

        match f.0.name.as_ref() {
            "symbol" => Ok(json!(encode_bytes(&ethereum_abi::Value::encode(&[
                ethereum_abi::Value::String(
                    TOKEN_METADATA
                        .get(&to)
                        .ok_or(PARSE_ERROR)?
                        .symbol
                        .to_string()
                )
            ])))),
            "decimals" => Ok(json!(encode_bytes(&ethereum_abi::Value::encode(&[
                ethereum_abi::Value::Uint(
                    ethabi::ethereum_types::U256::try_from(
                        TOKEN_METADATA.get(&to).ok_or(PARSE_ERROR)?.decimals // 0
                    )
                    .unwrap(),
                    256
                )
            ])))),
            "balanceOf" => {
                let mut db = aquire_db_read_lock!();
                let address = if let ethereum_abi::Value::Address(address) = f.1[0].value {
                    Address(address.as_bytes().try_into().unwrap())
                } else {
                    panic!("")
                };

                let balance =
                    ellipticoin_contracts::Token::get_underlying_balance(&mut db, address, to);

                Ok(encode_token_amount(balance))
            }
            _ => Ok(json!(null)),
        }
        // println!("token")
    } else {
        Ok(json!(null))
    }
}
