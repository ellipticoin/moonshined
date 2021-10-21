mod ellipticoin_abi;
pub mod erc20_abi;
mod signature_hashes;

use crate::constants::ELLIPTICOIN_DECIMALS;
use byte_slice_cast::AsByteSlice;
use ellipticoin_abi::ELLIPTICOIN_ABI;
use ellipticoin_contracts::token::tokens::TOKEN_METADATA;
use ellipticoin_contracts::token::tokens::USD;
use ellipticoin_contracts::{
    bridge::{EthereumMessage, PolygonMessage},
    system::Action,
};
use ellipticoin_types::{Address, Uint};

use num_bigint::BigUint;
use num_traits::pow;
use num_traits::ToPrimitive;
use std::{
    convert::{TryFrom, TryInto},
    fmt,
};

pub type Result<T> = std::result::Result<T, AbiError>;

#[derive(Debug, Clone)]
pub struct AbiError;

impl fmt::Display for AbiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "abi error")
    }
}

pub fn encode_action(action: &Action) -> Vec<u8> {
    match action {
        Action::AddLiquidity(amount, token) => vec![
            signature_hashes::ADD_LIQUIDITY.to_vec(),
            serde_eth::to_vec(&((*amount).as_i64(), serde_eth::Address(token.into()))).unwrap(),
        ],
        Action::Buy(underlying_input_amount, token, minimum_underlying_output_amount) => vec![
            signature_hashes::BUY.to_vec(),
            serde_eth::to_vec(&(
                (*underlying_input_amount).as_i64(),
                serde_eth::Address(token.into()),
                (*minimum_underlying_output_amount).as_i64(),
            ))
            .unwrap(),
        ],
        Action::CreatePool(amount, token, initial_price) => vec![
            signature_hashes::CREATE_POOL.to_vec(),
            serde_eth::to_vec(&(
                (*amount).as_i64(),
                serde_eth::Address(token.into()),
                (*initial_price).as_i64(),
            ))
            .unwrap(),
        ],
        Action::Pay(recipient, underlying_amount, token) => vec![
            signature_hashes::TRANSFER.to_vec(),
            serde_eth::to_vec(&(
                serde_eth::Address(recipient.into()),
                serde_eth::U256(scale_up_token_decimals(*underlying_amount, *token).unwrap()),
            ))
            .unwrap(),
        ],
        Action::RemoveLiquidity(percentage, token) => vec![
            signature_hashes::REMOVE_LIQUIDITY.to_vec(),
            serde_eth::to_vec(&((*percentage).as_i64(), serde_eth::Address(token.into()))).unwrap(),
        ],
        Action::Sell(underlying_input_amount, token, minimum_underlying_output_amount) => vec![
            signature_hashes::SELL.to_vec(),
            serde_eth::to_vec(&(
                (*underlying_input_amount).as_i64(),
                serde_eth::Address(token.into()),
                (*minimum_underlying_output_amount).as_i64(),
            ))
            .unwrap(),
        ],
        Action::Seal(onion_skin) => vec![
            signature_hashes::SEAL.to_vec(),
            serde_eth::to_vec(&(onion_skin)).unwrap(),
        ],
        Action::StartMining(host, onion_skin, layer_count) => vec![
            signature_hashes::START_MINING.to_vec(),
            serde_eth::to_vec(&(host, onion_skin, layer_count)).unwrap(),
        ],
        Action::ProcessPolygonMessages(messages, block_number) => vec![
            signature_hashes::PROCESS_POLYGON_MESSAGES.to_vec(),
            ethabi::encode(
                &[
                    ethabi::Token::Array(
                        messages
                            .iter()
                            .map(|message| match message {
                                PolygonMessage::Deposit(amount, token, address) => {
                                    ethabi::Token::Tuple(vec![
                                        ethabi::Token::Uint((*amount).into()),
                                        ethabi::Token::Address(token.0.into()),
                                        ethabi::Token::Address(address.0.into()),
                                        ethabi::Token::Int(0.into()),
                                        ethabi::Token::FixedBytes(vec![0; 32]),
                                    ])
                                }
                                PolygonMessage::ProcessWithdrawl(
                                    withdrawl_id,
                                    transaction_hash,
                                ) => ethabi::Token::Tuple(vec![
                                    ethabi::Token::Uint((0).into()),
                                    ethabi::Token::Address([0; 20].into()),
                                    ethabi::Token::Address([0; 20].into()),
                                    ethabi::Token::Int((*withdrawl_id).into()),
                                    ethabi::Token::FixedBytes(transaction_hash.to_vec().into()),
                                ]),
                            })
                            .collect::<Vec<ethabi::Token>>(),
                    ),
                    ethabi::Token::Uint(<u64>::try_from(*block_number).unwrap().into()),
                ][..],
            ),
        ],
        Action::ProcessEthereumMessages(messages, block_number) => vec![
            signature_hashes::PROCESS_ETHEREUM_MESSAGES.to_vec(),
            ethabi::encode(
                &[
                    ethabi::Token::Array(
                        messages
                            .iter()
                            .map(|message| match message {
                                EthereumMessage::SetUSDExchangeRate(amount) => {
                                    ethabi::Token::Tuple(vec![ethabi::Token::Uint(
                                        (*amount).to_bytes_le()[..].into(),
                                    )])
                                }
                            })
                            .collect::<Vec<ethabi::Token>>(),
                    ),
                    ethabi::Token::Uint(<u64>::try_from(*block_number).unwrap().into()),
                ][..],
            ),
        ],
        Action::CreateWithdrawlRequest(value, token) => vec![
            ELLIPTICOIN_ABI.functions[3].method_id().to_vec(),
            ethereum_abi::Value::encode(&[encode(*value), encode(*token)]),
        ],
        action => panic!("unknown action {:?}", action),
    }
    .concat()
}

pub fn decode_action(to: &[u8], value: &[u8], data: &[u8]) -> Result<Action> {
    if data.len() > 0 {
        decode_transcation_data(Address::try_from(to).map_err(|_| AbiError)?, data)
    } else {
        Ok(Action::Pay(
            Address::try_from(to).map_err(|_| AbiError)?,
            decode_transcation_value(value),
            USD,
        ))
    }
}
pub fn decode_transcation_value(value: &[u8]) -> Uint {
    Uint::try_from(
        (BigUint::from_bytes_be(value)
            / BigUint::from(pow(
                BigUint::from(10u32),
                18 as usize - *ELLIPTICOIN_DECIMALS,
            )))
        .to_u64()
        .unwrap(),
    )
    .unwrap()
}
pub fn decode_transcation_data(to: Address, data: &[u8]) -> Result<Action> {
    let f = ELLIPTICOIN_ABI
        .decode_input_from_slice(data)
        .map_err(|_| AbiError)?;
    match f.0.name.as_ref() {
        "addLiquidity" => Ok(Action::AddLiquidity(
            decode(&f.1[0].value)?,
            decode(&f.1[1].value)?,
        )),
        "buy" => Ok(Action::Buy(
            decode(&f.1[0].value)?,
            decode(&f.1[1].value)?,
            decode(&f.1[2].value)?,
        )),
        "createPool" => Ok(Action::CreatePool(
            decode(&f.1[0].value)?,
            decode(&f.1[1].value)?,
            decode(&f.1[2].value)?,
        )),
        "createWithdrawlRequest" => Ok(Action::CreateWithdrawlRequest(
            decode(&f.1[0].value)?,
            decode(&f.1[1].value)?,
        )),
        "processEthereumMessages" => Ok(Action::ProcessEthereumMessages(
            vec![],
            decode(&f.1[0].value)?,
        )),
        "processPolygonMessages" => Ok(Action::ProcessPolygonMessages(
            vec![],
            decode(&f.1[0].value)?,
        )),
        "pay" => Ok(Action::Pay(
            decode(&f.1[0].value)?,
            decode(&f.1[1].value)?,
            to,
        )),
        "sell" => Ok(Action::Sell(
            decode(&f.1[0].value)?,
            decode(&f.1[1].value)?,
            decode(&f.1[2].value)?,
        )),
        "removeLiquidity" => Ok(Action::RemoveLiquidity(
            decode(&f.1[0].value)?,
            decode(&f.1[1].value)?,
        )),
        "seal" => Ok(Action::Seal(decode(&f.1[0].value)?)),
        "startMining" => Ok(Action::StartMining(
            decode(&f.1[0].value)?,
            decode(&f.1[1].value)?,
            decode(&f.1[2].value)?,
        )),
        "transfer" => Ok(Action::Pay(
            decode(&f.1[0].value)?,
            decode_scaled_uint(&f.1[1].value, to)?,
            to,
        )),
        _ => Err(AbiError),
    }
}

fn encode<'de, E: Encodable<'de>>(value: E) -> ethereum_abi::Value {
    Encodable::encode(value)
}

trait Encodable<'de>: Sized {
    fn encode(value: Self) -> ethereum_abi::Value;
}

impl Encodable<'_> for Uint {
    fn encode(n: Self) -> ethereum_abi::Value {
        ethereum_abi::Value::Uint(
            ethabi::ethereum_types::U256::from_little_endian(&n.to_le_bytes()),
            64,
        )
    }
}

impl Encodable<'_> for Address {
    fn encode(address: Self) -> ethereum_abi::Value {
        ethereum_abi::Value::Address(ethabi::ethereum_types::H160(address.0))
    }
}

fn decode_scaled_uint(value: &ethereum_abi::Value, token: Address) -> Result<Uint> {
    if let ethereum_abi::Value::Uint(u256, 256) = value {
        scale_down_token_decimals(BigUint::from_bytes_le(u256.as_byte_slice()), token)
    } else {
        Err(AbiError)
    }
}

fn scale_up_token_decimals(value: Uint, token: Address) -> Result<BigUint> {
    Ok(scale_uint(
        &BigUint::from(value.as_i64() as u64),
        TOKEN_METADATA.get(&token).ok_or(AbiError)?.decimals as isize
            - *ELLIPTICOIN_DECIMALS as isize,
    ))
}
fn scale_down_token_decimals(value: BigUint, token: Address) -> Result<Uint> {
    Ok(Uint::try_from(
        scale_uint(
            &value,
            *ELLIPTICOIN_DECIMALS as isize
                - TOKEN_METADATA.get(&token).ok_or(AbiError)?.decimals as isize,
        )
        .to_u64()
        .ok_or(AbiError)?,
    )
    .map_err(|_| AbiError)?)
}

fn scale_uint(value: &BigUint, scale: isize) -> BigUint {
    if scale == 0 {
        value.clone()
    } else if scale > 0 {
        value * BigUint::from(pow(BigUint::from(10u8), scale.abs() as usize))
    } else {
        value / BigUint::from(pow(BigUint::from(10u8), scale.abs() as usize))
    }
}

fn decode<'de, D: Decodable<'de>>(value: &ethereum_abi::Value) -> Result<D> {
    Decodable::decode(value)
}

trait Decodable<'de>: Sized {
    fn decode(value: &ethereum_abi::Value) -> Result<Self>;
}

impl Decodable<'_> for String {
    fn decode(value: &ethereum_abi::Value) -> Result<Self> {
        if let ethereum_abi::Value::String(s) = value {
            Ok(s.to_string())
        } else {
            Err(AbiError)
        }
    }
}

impl Decodable<'_> for [u8; 32] {
    fn decode(value: &ethereum_abi::Value) -> Result<Self> {
        if let ethereum_abi::Value::FixedBytes(bytes) = value {
            Ok(bytes[..].try_into().map_err(|_| AbiError)?)
        } else {
            Err(AbiError)
        }
    }
}

impl Decodable<'_> for Address {
    fn decode(value: &ethereum_abi::Value) -> Result<Self> {
        if let ethereum_abi::Value::Address(h160) = value {
            Ok(Address(h160.0))
        } else {
            Err(AbiError)
        }
    }
}

impl Decodable<'_> for Uint {
    fn decode(value: &ethereum_abi::Value) -> Result<Self> {
        if let ethereum_abi::Value::Int(u256, _) = value {
            Uint::try_from(i64::from_le_bytes(
                u256.as_byte_slice()[0..8].try_into().unwrap(),
            ))
            .map_err(|_| AbiError)
        } else {
            Err(AbiError)
        }
    }
}
