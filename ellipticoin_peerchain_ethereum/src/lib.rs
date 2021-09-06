mod bridge_abi;

pub mod abi;
pub mod constants;
pub mod crypto;
pub mod helpers;
pub mod json_rpc;
pub mod rlp;
pub mod signature;
pub use lazy_static::lazy_static;

use crate::constants::{EXCHANGE_RATE_CURRENT_SELECTOR, POLL_INTERVAL, PROVIDER, USD_ADDRESS};
use async_std::{prelude::*, stream};
use ellipticoin_contracts::bridge::EthereumMessage;
use futures::stream::StreamExt;
use num_bigint::BigUint;
use std::sync::atomic::{AtomicU64, Ordering};

static BLOCK_NUMBER: AtomicU64 = AtomicU64::new(0);

pub fn event_stream(latest_block: u64) -> impl Stream<Item = (Vec<EthereumMessage>, u64)> {
    BLOCK_NUMBER.store(latest_block, Ordering::Relaxed);
    Box::pin(
        stream::once(())
            .chain(stream::interval(*POLL_INTERVAL))
            .filter_map(move |_| async move { poll(latest_block).await }),
    )
}

pub async fn poll(latest_block: u64) -> Option<(Vec<EthereumMessage>, u64)> {
    let current_block = PROVIDER.get_current_block().await.unwrap();
    if current_block == BLOCK_NUMBER.load(Ordering::Relaxed) {
        None
    } else {
        // If we're greater than 128 blocks behind assume there was a restart
        // and skip to the current block.
        if current_block - latest_block > 128 {
            current_block
        } else {
            BLOCK_NUMBER.load(Ordering::Relaxed) + 1
        };

        let usd_exchange_rate = BigUint::from_bytes_be(
            &PROVIDER
                .call(&USD_ADDRESS, &EXCHANGE_RATE_CURRENT_SELECTOR, current_block)
                .await
                .unwrap(),
        );

        BLOCK_NUMBER.store(latest_block, Ordering::Relaxed);
        Some((
            vec![EthereumMessage::SetUSDExchangeRate(usd_exchange_rate)],
            current_block,
        ))
    }
}
