use ellipticoin_contracts::constants::{BASE_FACTOR, BTC, C_DAI, DAI, ETH, LEVERAGED_BASE_TOKEN};
use ellipticoin_types::Address;
use hex_literal::hex;
use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref ELLIPTICOIN_DECIMALS: usize = BASE_FACTOR.to_string().len() - 1;
    pub static ref WEB3_URL: String = env::var("WEB3_URL").expect("WEB3_URL not set");
}

pub static BASE_TOKEN_UNDERLYING_DECIMALS: usize = 28;
pub static REDEEM_TIMEOUT: u64 = 30;
pub const REDEEM_TOPIC: [u8; 32] =
    hex!("ff051e185ca4ab867487cbb2112ad9dcf4b6e45ec93c6c83fe371bfd126d1da6");
pub const LEVERAGED_BASE_TOKEN_ADDRESS: Address = C_DAI;
pub const EXCHANGE_RATE_CURRENT_SELECTOR: [u8; 4] = hex!("bd6d894d");
pub const SUPPLY_RATE_PER_BLOCK_SELECTOR: [u8; 4] = hex!("ae9d70b0");
pub const TOKENS: [Address; 4] = [DAI, BTC, ETH, LEVERAGED_BASE_TOKEN];
pub const BRIDGE_ADDRESS: Address = Address(hex!("E55faDE7825Ad88581507C51c9f1b33827AaE5E8"));
pub const SAFE_ADDRESS: Address = Address(hex!("5510f178A57C4f4B456d747CdbfcD0A5b1b5473b"));
pub const ETH_ADDRESS: Address = Address(hex!("0000000000000000000000000000000000000000"));
