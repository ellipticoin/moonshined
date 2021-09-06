use crate::{helpers::eth_address, json_rpc::Provider};
use ellipticoin_contracts::constants::BASE_FACTOR;
use ellipticoin_types::Address;
use hex_literal::hex;
use k256::ecdsa::SigningKey;
use lazy_static::lazy_static;
use std::{env, time::Duration};

lazy_static! {
    pub static ref PRIVATE_KEY: SigningKey = {
        SigningKey::from_bytes(
            &hex::decode(&env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set"))
                .expect("Invalid PRIVATE_KEY"),
        )
        .unwrap()
    };
    pub static ref ADDRESS: Address = eth_address(&PRIVATE_KEY.verifying_key());
    pub static ref ELLIPTICOIN_DECIMALS: usize = BASE_FACTOR.to_string().len() - 1;
    pub static ref ETHEREUM_WEB3_URL: String =
        env::var("ETHEREUM_WEB3_URL").expect("ETHEREUM_WEB3_URL not set");
    pub static ref POLYGON_WEB3_URL: String =
        env::var("POLYGON_WEB3_URL").expect("POLYGON_WEB3_URL not set");
    pub static ref PROVIDER: Provider<'static> = Provider::new(&ETHEREUM_WEB3_URL);
    pub static ref POLL_INTERVAL: Duration = Duration::from_secs(900);
}

pub static BASE_TOKEN_UNDERLYING_DECIMALS: usize = 28;
pub static REDEEM_TIMEOUT: u64 = 30;
pub const REDEEM_TOPIC: [u8; 32] =
    hex!("ff051e185ca4ab867487cbb2112ad9dcf4b6e45ec93c6c83fe371bfd126d1da6");
pub const TRANSFER_TOPIC: [u8; 32] =
    hex!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");
pub const EXCHANGE_RATE_CURRENT_SELECTOR: [u8; 4] = hex!("bd6d894d");
pub const BRIDGE_ADDRESS: Address = Address(hex!("53507c188a5b1bfca4ed27f45ae8b2e2324ed24d"));
pub const USD_ADDRESS: Address = Address(hex!("39aa39c021dfbae8fac545936693ac917d5e7563"));
pub const SAFE_ADDRESS: Address = Address(hex!("DE892F0Ba04B3923A01a8384C947a346BA296197"));
