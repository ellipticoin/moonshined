use crate::constants::WEB_SOCKET_BROADCASTER;
use crate::{
    db,
    transaction::{self, run},
};
use async_std::task::spawn;
use ellipticoin_contracts::bridge::{EthereumMessage, PolygonMessage};
use ellipticoin_contracts::Action;
use ellipticoin_types::Uint;
use futures::StreamExt;
use std::convert::TryFrom;

// pub async fn poll_ethereum() {

pub fn start_polling() {
    spawn(async move {
        let ethereum_block_number = db::get_ethereum_block_number().await;
        ellipticoin_peerchain_ethereum::event_stream(ethereum_block_number)
            .for_each(|(messages, block_number)| process_ethereum_messages(messages, block_number))
            .await;
    });
    spawn(async move {
        let polygon_block_number = db::get_polygon_block_number().await;
        ellipticoin_peerchain_polygon::event_stream(polygon_block_number)
            .for_each(|(messages, block_number)| process_polygon_messages(messages, block_number))
            .await;
    });
}
pub async fn process_ethereum_messages(messages: Vec<EthereumMessage>, new_block_number: u64) {
    let ethereum_block_number = db::get_ethereum_block_number().await;
    if ethereum_block_number + 1 == new_block_number {
        println!("Processed Ethereum Block #{}", ethereum_block_number + 1);
    } else {
        println!(
            "Processed Ethereum Block #{}-#{}",
            ethereum_block_number + 1,
            new_block_number
        );
    }
    run(transaction::sign(Action::ProcessEthereumMessages(
        messages.clone(),
        Uint::try_from(new_block_number).unwrap(),
    ))
    .await)
    .await
    .unwrap();
    if messages.len() > 0 {
        WEB_SOCKET_BROADCASTER.broadcast().await
    }
}

pub async fn process_polygon_messages(messages: Vec<PolygonMessage>, new_block_number: u64) {
    let block_number = db::get_polygon_block_number().await;
    if block_number + 1 == new_block_number {
        println!("Processed Polygon Block #{}", block_number + 1);
    } else {
        println!(
            "Processed Polygon Block #{}-#{}",
            block_number + 1,
            new_block_number
        );
    }
    run(transaction::sign(Action::ProcessPolygonMessages(
        messages.clone(),
        Uint::try_from(new_block_number).unwrap(),
    ))
    .await)
    .await
    .unwrap();
    if messages.len() > 0 {
        WEB_SOCKET_BROADCASTER.broadcast().await
    }
}
