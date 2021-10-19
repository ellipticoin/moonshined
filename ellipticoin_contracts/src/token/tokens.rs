use crate::define_token;
use ellipticoin_types::Address;
use std::collections::HashMap;

define_token!(WBTC, "1bfd67037b42cf73acf2047067bd4f2c47d9bfd6");
define_token!(ETH, "7ceb23fd6bc0add59e62ac25578270cff1b9f619");
define_token!(MSX, "d604b56B3d741e5CF83791a62FB256e6fac943c1");
define_token!(CUSDC, "d871b40646e1a6dbded6290b6b696459a69c68a0");
define_token!(MATIC, "0d500b1d8e8ef31e21c99d1db9a6444d3adf1270");
define_token!(COMP, "8505b9d2254a7ae468c0e9dd10ccea3a837aef5c");
define_token!(SOL, "7DfF46370e9eA5f0Bad3C4E29711aD50062EA7A4");
define_token!(LINK, "53e0bca35ec356bd5dddfebbd1fc0fd03fabad39");
define_token!(QUICK, "831753dd7087cac61ab5644b308642cc1c33dc13");
define_token!(AAVE, "d6df932a45c0f255f85145f286ea0b292b21c90b");
define_token!(UNI, "b33eaad8d922b1083446dc23f610c2567fb5180f");

lazy_static! {
    pub static ref TOKEN_DECIMALS: HashMap<Address, u8> =
        vec![(CUSDC, 8), (MSX, 6), (WBTC, 8), (ETH, 18), (MATIC, 18),]
            .into_iter()
            .collect();
    pub static ref TOKENS: [Address; 5] = [WBTC, MSX, ETH, CUSDC, COMP];
}
pub const USD: Address = CUSDC;
pub const INCENTIVISED_POOLS: [Address; 2] = [WBTC, ETH];
