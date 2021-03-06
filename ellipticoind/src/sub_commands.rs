use crate::{api, config::socket, db, miner, peerchains, start_up};
use ellipticoin_peerchain_ethereum::signature::eth_address;
use k256::ecdsa::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;

pub fn generate_keypair() {
    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = VerifyingKey::from(&signing_key);
    println!(
        "Verifing Key (Address): {}",
        hex::encode(eth_address(&verifying_key).0)
    );
    println!("Signing Key: {}", hex::encode(signing_key.to_bytes()));
}

pub async fn main() {
    db::initialize().await;
    start_up::reset_state().await;
    start_up::catch_up().await;
    start_up::start_miner().await;
    peerchains::start_polling();
    api::start(socket()).await;
    miner::run().await;
}
