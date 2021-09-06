#[macro_export]
macro_rules! address {
    ( $hex:expr ) => {
        ethereum_types::Address::from(hex!($hex))
    };
}
