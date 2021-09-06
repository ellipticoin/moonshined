use hex_literal::hex;
pub const ADD_LIQUIDITY: [u8; 4] = hex!("1ee8621d"); // createPool(int64,address)
pub const BUY: [u8; 4] = hex!("09d351a0"); // buy(int64,address,int64,address)
pub const CREATE_POOL: [u8; 4] = hex!("9f2c4c6f"); // createPool(int64,address,int64)
pub const PROCESS_POLYGON_MESSAGES: [u8; 4] = hex!("00d961d1"); // processPolygonMessages((uint8,int64,address,address,int64,bytes32)[])
pub const REMOVE_LIQUIDITY: [u8; 4] = hex!("e47f9ade"); // createPool(int64,address)
pub const SEAL: [u8; 4] = hex!("b07eeda8"); // seal(bytes32)
pub const SELL: [u8; 4] = hex!("255f7e5b"); // sell(int64,address,int64,address)
pub const START_MINING: [u8; 4] = hex!("cc0b4376"); // startMining(string,bytes32,int64)