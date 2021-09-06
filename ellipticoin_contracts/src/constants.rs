use ellipticoin_types::Address;
use std::collections::HashMap;
use std::time::Duration;

lazy_static! {
    pub static ref TOKEN_DECIMALS: HashMap<Address, u8> =
        vec![(CUSDC, 8), (MSX, 6), (WBTC, 8), (ETH, 18), (MATIC, 18),]
            .into_iter()
            .collect();
}

pub const BASE_TOKEN_MANTISSA: usize = 6;
pub const EXCHANGE_RATE_MANTISSA: usize = 10;
pub const WBTC: Address = Address(hex!("1bfd67037b42cf73acf2047067bd4f2c47d9bfd6"));
pub const ETH: Address = Address(hex!("7ceb23fd6bc0add59e62ac25578270cff1b9f619"));
pub const MSX: Address = Address(hex!("d604b56B3d741e5CF83791a62FB256e6fac943c1"));
pub const CUSDC: Address = Address(hex!("d871b40646e1a6dbded6290b6b696459a69c68a0"));
pub const MATIC: Address = Address(hex!("0d500b1d8e8ef31e21c99d1db9a6444d3adf1270"));
pub const BASE_FACTOR: u64 = 1_000_000;
pub const FEE: u64 = 3_000;
pub const BASE_TOKEN: Address = CUSDC;
pub const USD: Address = CUSDC;
pub const INCENTIVISED_POOLS: [Address; 2] = [WBTC, ETH];
pub const MINER_ALLOW_LIST: [Address; 2] = [
    Address(hex!("0113713f91dd6a7c179a038e66e5919a9a0a9d1d")),
    Address(hex!("418b993b7d17b45937ef4f69a06a3433cd30b5ce")),
];
pub const RATIFICATION_THRESHOLD: u64 = 20;

lazy_static! {
    pub static ref BLOCK_TIME: Duration = Duration::from_secs(4);
    pub static ref TOKENS: [Address; 4] = [WBTC, MSX, ETH, CUSDC];
}
