pub mod constants;
pub mod transaction;
use ellipticoin_types::Address;
pub use transaction::*;

use crate::constants::{
    BRIDGE_ADDRESS, ELLIPTICOIN_DECIMALS, ETH_ADDRESS, EXCHANGE_RATE_CURRENT_SELECTOR,
    LEVERAGED_BASE_TOKEN_ADDRESS, REDEEM_TOPIC, SAFE_ADDRESS, SUPPLY_RATE_PER_BLOCK_SELECTOR,
    TOKENS, WEB3_URL,
};
use ellipticoin_contracts::{
    bridge::{Mint, Redeem, Update},
    constants::BASE_FACTOR,
};
use num_bigint::BigUint;
use num_traits::{pow::pow, ToPrimitive};
use serde::{de::Deserializer, Deserialize};
use serde_json::{json, Value};
use std::{collections::HashMap, convert::TryInto, task::Poll};
use surf;
pub use transaction::ecrecover;

#[derive(Deserialize)]
struct TransfersResponse {
    result: TransfersResult,
}

#[derive(Deserialize)]
struct TransfersResult {
    transfers: Vec<Transfer>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Transfer {
    hash: String,
    asset: String,
    #[serde(deserialize_with = "parse_address")]
    from: Option<Address>,
    raw_contract: RawContract,
}

#[derive(Deserialize, Debug)]
struct RawContract {
    #[serde(deserialize_with = "parse_address")]
    address: Option<Address>,
    #[serde(deserialize_with = "parse_big_uint")]
    value: BigUint,
    #[serde(deserialize_with = "parse_usize")]
    decimal: usize,
}

#[derive(Deserialize, Debug)]
struct RedeemLog {
    #[serde(deserialize_with = "parse_big_uint")]
    data: BigUint,
}

pub async fn poll(latest_block: u64) -> Result<Poll<Update>, surf::Error> {
    let current_block = get_current_block().await?;
    if current_block == latest_block {
        Ok(Poll::Pending)
    } else {
        let base_token_exchange_rate = eth_call(
            LEVERAGED_BASE_TOKEN_ADDRESS,
            EXCHANGE_RATE_CURRENT_SELECTOR,
            current_block,
        )
        .await?;
        let base_token_interest_rate = get_base_token_interest_rate(current_block).await.unwrap();

        // Ethereum nodes only store 128 blocks of history.
        // If we're greater than 128 blocks behind assume there was a restart
        // and skip to the current block.
        let from_block = if current_block - latest_block > 128 {
            current_block
        } else {
            latest_block + 1
        };
        Ok(Poll::Ready(Update {
            block_number: current_block,
            base_token_interest_rate,
            base_token_exchange_rate,
            mints: [
                get_mints_to_bridge(from_block, current_block)
                    .await
                    .unwrap(),
                get_mints_to_safe(from_block, current_block).await.unwrap(),
            ]
            .concat(),
            redeems: get_redeems(from_block, current_block).await?,
        }))
    }
}

async fn get_mints_to_bridge(from_block: u64, to_block: u64) -> Result<Vec<Mint>, surf::Error> {
    let mints = get_asset_transfers(BRIDGE_ADDRESS, from_block, to_block)
        .await?
        .iter()
        .filter_map(|transfer: &Transfer| {
            if transfer.from.unwrap() == SAFE_ADDRESS {
                None
            } else {
                Some(Mint(
                    scale_down(
                        transfer.raw_contract.value.clone(),
                        transfer.raw_contract.decimal,
                    ),
                    transfer.raw_contract.address.unwrap_or(ETH_ADDRESS),
                    transfer.from.unwrap(),
                ))
            }
        })
        .collect();
    Ok(mints)
}

async fn get_mints_to_safe(from_block: u64, to_block: u64) -> Result<Vec<Mint>, surf::Error> {
    let mints = get_asset_transfers(SAFE_ADDRESS, from_block, to_block)
        .await?
        .iter()
        .map(|transfer: &Transfer| {
            Mint(
                scale_down(
                    transfer.raw_contract.value.clone(),
                    transfer.raw_contract.decimal,
                ),
                transfer.raw_contract.address.unwrap_or(ETH_ADDRESS),
                transfer.from.unwrap(),
            )
        })
        .collect();
    Ok(mints)
}

async fn get_asset_transfers(
    to_address: Address,
    from_block: u64,
    to_block: u64,
) -> Result<Vec<Transfer>, surf::Error> {
    loop {
        let mut res = surf::post(WEB3_URL.clone())
            .body(json!(
             {
             "id": 1,
             "jsonrpc": "2.0",
             "method": "alchemy_getAssetTransfers",
             "params": [{
             "fromBlock": format!("0x{}", BigUint::from(from_block).to_str_radix(16)),
             "toBlock": format!("0x{}", BigUint::from(to_block).to_str_radix(16)),
             "toAddress": format!("0x{}", hex::encode(to_address)),
             "contractAddresses": TOKENS.iter().map(|token|
                     format!("0x{}", hex::encode(&token))
                     ).collect::<Vec<String>>()
             }]
             }
            ))
            .await
            .unwrap();
        let transfers_response_string = res.body_string().await?;
        let transfers_response =
            match serde_json::from_str::<TransfersResponse>(&transfers_response_string) {
                Ok(transfers_response) => transfers_response.result.transfers,
                Err(err) => {
                    println!("{}: {}", err.to_string(), transfers_response_string);
                    continue;
                }
            };
        return Ok(transfers_response);
    }
}

async fn get_redeems(from_block: u64, to_block: u64) -> Result<Vec<Redeem>, surf::Error> {
    let logs = get_logs(BRIDGE_ADDRESS, from_block, to_block, vec![REDEEM_TOPIC]).await?;
    Ok(logs
        .iter()
        .cloned()
        .map(|log| {
            let redeem_log: RedeemLog = serde_json::value::from_value(log).unwrap();
            let redeem_id = redeem_log.data.to_u64().unwrap();
            Redeem(redeem_id)
        })
        .collect())
}

fn scale_down(amount: BigUint, decimals: usize) -> u64 {
    (amount / BigUint::from(pow(BigUint::from(10u32), decimals - *ELLIPTICOIN_DECIMALS)))
        .to_u64()
        .unwrap()
}

pub async fn get_base_token_interest_rate(block_number: u64) -> Result<u64, surf::Error> {
    let rate = eth_call(
        LEVERAGED_BASE_TOKEN_ADDRESS,
        SUPPLY_RATE_PER_BLOCK_SELECTOR,
        block_number,
    )
    .await
    .unwrap();
    let mantissa = pow(10f64, 18);
    let blocks_per_day = 4 * 60 * 24;
    let days_per_year = 365;
    let apy_as_percentage = ((pow(
        (rate.to_u64().unwrap() as f64 / mantissa as f64 * blocks_per_day as f64) + 1f64,
        days_per_year,
    )) - 1f64)
        * 100f64;
    Ok((apy_as_percentage * (BASE_FACTOR as f64)) as u64)
}

pub async fn eth_call(
    contract_address: Address,
    selector: [u8; 4],
    block_number: u64,
) -> Result<BigUint, surf::Error> {
    let res_hex = loop {
        let mut res = match surf::post(WEB3_URL.clone())
            .body(json!(
             {
             "id": 1,
             "jsonrpc": "2.0",
             "method": "eth_call",
             "params": [
                 {
                     "to": format!("0x{}", hex::encode(contract_address)),
                     "data": format!("0x{}", hex::encode(selector)),
                 },
                 format!("0x{}", BigUint::from(block_number).to_str_radix(16))
             ]}
            ))
            .await
        {
            Ok(res_hex) => res_hex,
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };
        let res_hash_map = match res.body_json::<HashMap<String, serde_json::Value>>().await {
            Ok(res_hash_map) => res_hash_map,
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };
        if res_hash_map.contains_key("result") {
            break serde_json::from_value::<String>(res_hash_map.get("result").unwrap().clone())?;
        }
    };

    Ok(BigUint::parse_bytes(res_hex.trim_start_matches("0x").as_bytes(), 16).unwrap())
}

pub async fn get_current_block() -> Result<u64, surf::Error> {
    let res_hex = loop {
        let mut res = match surf::post(WEB3_URL.clone())
            .body(json!(
                {
                    "id": 1,
                    "jsonrpc": "2.0",
                    "method": "eth_blockNumber",
                    "params": []
                }
            ))
            .await
        {
            Ok(res) => res,
            Err(_) => continue,
        };
        let res_hashmap = match res.body_json::<HashMap<String, serde_json::Value>>().await {
            Ok(res_hashmap) => res_hashmap,
            Err(_) => continue,
        };
        let res_hex = match res_hashmap.get("result") {
            Some(result) => serde_json::from_value::<String>(result.clone()).unwrap(),
            None => continue,
        };
        if !(res_hex == "0x0") {
            break res_hex;
        }
    };

    Ok(
        BigUint::parse_bytes(res_hex.trim_start_matches("0x").as_bytes(), 16)
            .unwrap()
            .to_u64()
            .unwrap(),
    )
}

async fn get_logs(
    address: Address,
    from_block: u64,
    to_block: u64,
    topics: Vec<[u8; 32]>,
) -> Result<Vec<Value>, surf::Error> {
    loop {
        let mut res = match surf::post(WEB3_URL.clone())
        .body(json!(
            {
                "id": 1,
                "jsonrpc": "2.0",
                "method": "eth_getLogs",
                "params": [{
                    "address": format!("0x{}", hex::encode(address)),
                    "fromBlock": format!("0x{}", BigUint::from(from_block).to_str_radix(16)),
                    "toBlock": format!("0x{}", BigUint::from(to_block).to_str_radix(16)),
                    "topics": topics.iter().map(|topic| format!("0x{}", hex::encode(topic))).collect::<Vec<String>>(),
                }]
            }
        ))
        .await {
            Ok(res) => res,
            Err(_) => continue,
        };
        let body_json = res
            .body_json::<HashMap<String, serde_json::Value>>()
            .await?;
        let result = match body_json.get("result") {
            Some(res) => res.clone(),
            None => {
                print!("{:?}", body_json);
                continue;
            }
        };
        match serde_json::from_value(result) {
            Ok(res) => break Ok(res),
            Err(_) => continue,
        }
    }
}

fn parse_address<'de, D>(deserializer: D) -> Result<Option<Address>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    if value.is_null() {
        return Ok(None);
    }
    let s = value.as_str().unwrap();

    let bytes = hex::decode(s.trim_start_matches("0x")).unwrap();
    Ok(Some(Address(
        bytes[..][bytes.len() - 20..].try_into().unwrap(),
    )))
}

fn parse_big_uint<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    Ok(
        BigUint::parse_bytes(s.trim_start_matches("0x").as_bytes(), 16)
            .ok_or(serde::de::Error::custom("error parsing bytes"))?,
    )
}

fn parse_usize<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    parse_big_uint(deserializer)
        .map(|n| n.to_usize())?
        .ok_or(serde::de::Error::custom(
            "error converting BigUint to usize",
        ))
}
