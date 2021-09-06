use ellipticoin_types::Address;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use serde::{de::Deserializer, Deserialize};
use serde_json::{json, Value};
use std::{collections::HashMap, convert::TryInto};
#[derive(Deserialize, Debug)]
pub struct Log {
    #[serde(deserialize_with = "parse_address")]
    pub address: Address,
    #[serde(deserialize_with = "parse_topics")]
    pub topics: Vec<[u8; 32]>,
    #[serde(deserialize_with = "parse_bytes")]
    pub data: Vec<u8>,
}

#[derive(Deserialize)]
pub struct LogsResponse {
    pub result: Vec<Log>,
}

#[derive(Deserialize)]
pub struct TransactionsResponse {
    pub result: Block,
}

#[derive(Deserialize, Debug)]
pub struct Block {
    pub transactions: Vec<Transaction>,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    #[serde(deserialize_with = "parse_address")]
    pub from: Address,
    #[serde(deserialize_with = "parse_address_or_none", default)]
    pub to: Option<Address>,
    #[serde(deserialize_with = "parse_big_uint")]
    pub value: BigUint,
}

pub struct Provider<'a> {
    url: &'a str,
}

impl<'a> Provider<'a> {
    pub fn new(url: &'a str) -> Self {
        Self { url }
    }

    pub async fn call(
        &self,
        contract_address: &Address,
        data: &[u8],
        block_number: u64,
    ) -> Result<Vec<u8>, surf::Error> {
        let mut res = self
            .post(
                "call",
                json!([{
                     "to": encode_address(contract_address),
                     "data": encode_bytes(data),
                 },
                 encode_u64(block_number)]),
            )
            .await?;
        let body_json = res
            .body_json::<HashMap<String, serde_json::Value>>()
            .await?;
        let result = body_json.get("result").unwrap();
        Ok(hex::decode(
            serde_json::from_value::<String>(result.clone())
                .unwrap()
                .trim_start_matches("0x"),
        )
        .unwrap())
    }

    pub async fn send_raw_transaction(&self, data: &[u8]) -> Result<Option<[u8; 32]>, surf::Error> {
        let mut res = self
            .post("sendRawTransaction", json!([encode_bytes(data)]))
            .await?;
        let body_json = res
            .body_json::<HashMap<String, serde_json::Value>>()
            .await?;
        let result = body_json.get("result").unwrap();
        if result.is_null() {
            Ok(None)
        } else {
            Ok(Some(
                hex::decode(
                    serde_json::from_value::<String>(result.clone())
                        .unwrap()
                        .trim_start_matches("0x"),
                )
                .unwrap()
                .try_into()
                .unwrap(),
            ))
        }
    }

    pub async fn get_block_by_number(&self, block_number: u64) -> Result<Block, surf::Error> {
        let mut res = self
            .post("getBlockByNumber", json!([encode_u64(block_number), true]))
            .await?;
        let transactions_response_string = res.body_string().await?;
        Ok(
            serde_json::from_str::<TransactionsResponse>(&transactions_response_string)
                .map(|transactions_response| transactions_response.result)
                .unwrap(),
        )
    }

    pub async fn get_current_block(&self) -> Result<u64, surf::Error> {
        let mut res = self.post("blockNumber", json!([])).await?;
        let res_hashmap = res
            .body_json::<HashMap<String, serde_json::Value>>()
            .await?;
        let res_hex = res_hashmap.get("result").unwrap();

        Ok(BigUint::parse_bytes(
            serde_json::from_value::<String>(res_hex.clone())
                .unwrap()
                .trim_start_matches("0x")
                .as_bytes(),
            16,
        )
        .unwrap()
        .to_u64()
        .unwrap())
    }

    pub async fn get_logs(
        &self,
        from_block: u64,
        to_block: u64,
        address: Vec<Address>,
        topics: serde_json::Value,
    ) -> Result<Vec<Log>, surf::Error> {
        let mut res = self.post(
            "getLogs",
            json!([{
                    "address": address.iter().map(encode_address).collect::<Vec<      String>>(),
                    "fromBlock": encode_u64(from_block),
                    "toBlock": encode_u64(to_block),
                    "topics": topics,
}]),
        ).await?;
        let body_json = res
            .body_json::<HashMap<String, serde_json::Value>>()
            .await?;
        let result = body_json.get("result").unwrap();
        Ok(serde_json::from_value::<Vec<Log>>(result.clone()).unwrap())
    }

    pub async fn get_transaction_count(&self, address: &Address) -> Result<u64, surf::Error> {
        let mut res = self
            .post(
                "getTransactionCount",
                json!([encode_address(address), "latest"]),
            )
            .await?;
        let body_json = res
            .body_json::<HashMap<String, serde_json::Value>>()
            .await?;
        let result = body_json.get("result").unwrap();
        Ok(BigUint::parse_bytes(
            serde_json::from_value::<String>(result.clone())
                .unwrap()
                .trim_start_matches("0x")
                .as_bytes(),
            16,
        )
        .unwrap()
        .to_u64()
        .unwrap())
    }

    async fn post(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<surf::Response, surf::Error> {
        surf::post(self.url)
            .body(json!(
            {
            "id": 1,
            "jsonrpc": "2.0",
            "method": format!("eth_{}",method),
            "params": params
            }))
            .await
    }
}

fn parse_address<'de, D>(deserializer: D) -> Result<Address, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    let s = value.as_str().unwrap();

    let bytes = hex::decode(s.trim_start_matches("0x")).unwrap();
    Ok(Address(bytes[..][bytes.len() - 20..].try_into().unwrap()))
}

pub fn parse_topics<'de, D>(deserializer: D) -> Result<Vec<[u8; 32]>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    let s: Vec<Value> = value
        .as_array()
        .ok_or(serde::de::Error::custom("oops"))?
        .to_vec();
    s.iter()
        .map(|topic| {
            Ok(
                hex::decode(topic.as_str().unwrap().trim_start_matches("0x"))
                    .unwrap()
                    .try_into()
                    .unwrap(),
            )
        })
        .collect()
}

fn parse_address_or_none<'de, D>(deserializer: D) -> Result<Option<Address>, D::Error>
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

pub fn parse_big_uint<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    Ok(
        BigUint::parse_bytes(s.trim_start_matches("0x").as_bytes(), 16)
            .ok_or(serde::de::Error::custom("error parsing bytes"))?,
    )
}

pub fn parse_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    Ok(hex::decode(s.trim_start_matches("0x").as_bytes())
        .map_err(|_| serde::de::Error::custom("error parsing bytes"))?)
}

fn _parse_usize<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    parse_big_uint(deserializer)
        .map(|n| n.to_usize())?
        .ok_or(serde::de::Error::custom(
            "error converting BigUint to usize",
        ))
}

fn encode_address(address: &Address) -> String {
    encode_bytes(&address.0)
}

fn encode_bytes(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

pub fn encode_topic(address: &[u8; 32]) -> String {
    format!("0x{}", hex::encode(address))
}

pub fn encode_address_topic(address: Address) -> String {
    format!("0x{}", hex::encode(left_pad(&address.0, 32)))
}

fn left_pad(s: &[u8], length: usize) -> Vec<u8> {
    let mut buf = vec![0u8; length - s.len()];
    buf.extend_from_slice(&s[0..s.len()]);
    buf
}

fn encode_u64(n: u64) -> String {
    format!("0x{}", BigUint::from(n).to_str_radix(16))
}
