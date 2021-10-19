use ellipticoin_types::Address;

use std::time::Duration;

pub const BASE_TOKEN_MANTISSA: usize = 6;
pub const EXCHANGE_RATE_MANTISSA: usize = 10;
pub const BASE_FACTOR: u64 = 1_000_000;
pub const FEE: u64 = 3_000;
pub const MINER_ALLOW_LIST: [Address; 2] = [
    Address(hex!("0113713f91dd6a7c179a038e66e5919a9a0a9d1d")),
    Address(hex!("418b993b7d17b45937ef4f69a06a3433cd30b5ce")),
];
pub const RATIFICATION_THRESHOLD: u64 = 20;

lazy_static! {
    pub static ref BLOCK_TIME: Duration = Duration::from_secs(4);
}
