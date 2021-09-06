mod bridge_abi;

pub mod abi;
pub mod constants;
pub mod rlp;
pub mod withdrawls;
pub use lazy_static::lazy_static;
pub use withdrawls::process_withdrawl;

use crate::constants::POLL_INTERVAL;
use crate::constants::{
    BRIDGE_ADDRESS, ELLIPTICOIN_DECIMALS, PROVIDER, SAFE_ADDRESS, TRANSFER_TOPIC,
};
use async_std::{prelude::*, stream};
use ellipticoin_contracts::{
    bridge::PolygonMessage,
    constants::{MATIC, TOKENS, TOKEN_DECIMALS},
};

use ellipticoin_peerchain_ethereum::json_rpc::Block;
use ellipticoin_peerchain_ethereum::json_rpc::{encode_address_topic, encode_topic};
use ellipticoin_types::Address;
use futures::stream::StreamExt;
use futures::stream::TryStreamExt;
use num_bigint::BigUint;
use num_traits::{pow::pow, ToPrimitive};
use serde_json::json;
use std::convert::TryInto;
use std::sync::atomic::{AtomicU64, Ordering};
use surf;

static BLOCK_NUMBER: AtomicU64 = AtomicU64::new(0);

pub fn event_stream(latest_block: u64) -> impl Stream<Item = (Vec<PolygonMessage>, u64)> {
    BLOCK_NUMBER.store(latest_block, Ordering::Relaxed);
    Box::pin(
        stream::once(())
            .chain(stream::interval(*POLL_INTERVAL))
            .filter_map(move |_| async move { poll(latest_block).await }),
    )
}

pub async fn poll(latest_block: u64) -> Option<(Vec<PolygonMessage>, u64)> {
    let current_block = PROVIDER.get_current_block().await.unwrap();
    if current_block == BLOCK_NUMBER.load(Ordering::Relaxed) {
        None
    } else {
        // If we're greater than 128 blocks behind assume there was a restart
        // and skip to the current block.
        let from_block = if current_block - latest_block > 128 {
            current_block
        } else {
            BLOCK_NUMBER.load(Ordering::Relaxed) + 1
        };

        let messages = vec![
            get_matic_deposits(from_block, current_block).await.unwrap(),
            get_token_deposits(from_block, current_block).await.unwrap(),
        ]
        .concat();
        BLOCK_NUMBER.store(latest_block, Ordering::Relaxed);
        if messages.len() > 0 {
            Some((messages, current_block))
        } else {
            None
        }
    }
}

async fn get_matic_deposits(
    from_block: u64,
    to_block: u64,
) -> Result<Vec<PolygonMessage>, surf::Error> {
    Ok(futures::stream::iter(from_block..=to_block)
        .then(|block_number| PROVIDER.get_block_by_number(block_number))
        .try_collect::<Vec<Block>>()
        .await?
        .into_iter()
        .flat_map(|block| block.transactions)
        .filter(|transaction| {
            transaction.to == Some(SAFE_ADDRESS) || transaction.to == Some(BRIDGE_ADDRESS)
        })
        .filter(|transaction| transaction.from != SAFE_ADDRESS)
        .map(|transaction| {
            println!(
                "{} {} {}",
                scale_down(transaction.value.clone(), 18u8),
                hex::encode(MATIC.0),
                hex::encode(transaction.from)
            );
            PolygonMessage::Deposit(scale_down(transaction.value, 18u8), MATIC, transaction.from)
        })
        .collect())
}

async fn get_token_deposits(
    from_block: u64,
    to_block: u64,
) -> Result<Vec<PolygonMessage>, surf::Error> {
    Ok(PROVIDER
        .get_logs(
            from_block,
            to_block,
            TOKENS.to_vec(),
            json!([
                encode_topic(&TRANSFER_TOPIC),
                null,
                [
                    encode_address_topic(BRIDGE_ADDRESS),
                    encode_address_topic(SAFE_ADDRESS)
                ]
            ]),
        )
        .await?
        .iter()
        .filter(|log| {
            let from = Address(log.topics[1][12..].try_into().unwrap());
            from != SAFE_ADDRESS
        })
        .map(|log| {
            let token = log.address;
            let from = Address(log.topics[1][12..].try_into().unwrap());
            let amount = BigUint::from_bytes_be(&log.data);
            let scaled_amount = scale_down(amount, *TOKEN_DECIMALS.get(&token).unwrap());
            println!(
                "{} {} {}",
                scaled_amount,
                hex::encode(token),
                hex::encode(from)
            );
            PolygonMessage::Deposit(scaled_amount, token, from)
        })
        .collect())
}

fn scale_down(amount: BigUint, decimals: u8) -> u64 {
    (amount
        / BigUint::from(pow(
            BigUint::from(10u32),
            decimals as usize - *ELLIPTICOIN_DECIMALS,
        )))
    .to_u64()
    .unwrap()
}
