use crate::{
    aquire_db_read_lock, aquire_db_write_lock,
    config::{verification_key, HOST, OPTS},
    constants::{DB, DEFAULT_GAS_LIMIT, TRANSACTIONS_FILE, TRANSACTION_QUEUE, TRANSFER_GAS_LIMIT},
    hash_onion,
};
use anyhow::Result;

use ellipticoin_contracts::{
    contract::Contract, token::tokens::CUSDC, Action, Bridge, Ellipticoin, System, Transaction, AMM,
};
use ellipticoin_peerchain_ethereum::{abi::encode_action, crypto, rlp, signature::Signature};

use ellipticoin_peerchain_polygon::process_withdrawl;
use ellipticoin_types::{
    db::{Backend, Db},
    traits::Run,
    Address,
};
use num_bigint::BigUint;
use num_traits::{pow, FromPrimitive};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedTransaction(pub Transaction, pub Signature);

pub async fn sign(action: Action) -> SignedTransaction {
    let mut db = aquire_db_read_lock!();
    let mut transaction: SignedTransaction = SignedTransaction(
        Transaction {
            transaction_number: System::get_next_transaction_number(&mut db, verification_key()),
            action,
        },
        Default::default(),
    );
    transaction.sign();
    transaction
}

impl SignedTransaction {
    fn sign(&mut self) {
        self.1 = crypto::sign(&self.signing_data())
    }

    fn signing_data(&self) -> Vec<u8> {
        rlp::encode(vec![
            BigUint::from_u64(self.0.transaction_number)
                .unwrap()
                .to_bytes_be()
                .to_vec(),
            vec![],
            BigUint::from_u64(self.gas_limit())
                .unwrap()
                .to_bytes_be()
                .to_vec(),
            self.to().unwrap().0.to_vec(),
            self.value(),
            self.data(),
            BigUint::from_u64(OPTS.chain_id)
                .unwrap()
                .to_bytes_be()
                .to_vec(),
            vec![],
            vec![],
        ])
    }

    fn gas_limit(&self) -> u64 {
        if matches!(&self.0.action, Action::Pay(..)) {
            TRANSFER_GAS_LIMIT
        } else {
            DEFAULT_GAS_LIMIT
        }
    }
    fn to(&self) -> Option<Address> {
        Some(match &self.0.action {
            Action::Pay(recipient, _amount, token) => {
                if *token == CUSDC {
                    *recipient
                } else {
                    *token
                }
            }
            Action::CreatePool(..) => AMM::address(),
            Action::AddLiquidity(..) => AMM::address(),
            Action::Buy(..) => AMM::address(),
            Action::RemoveLiquidity(..) => AMM::address(),
            Action::CreateWithdrawlRequest(..) => Bridge::address(),
            Action::Seal(..) => Ellipticoin::address(),
            Action::Sell(..) => AMM::address(),
            Action::StartMining(..) => Ellipticoin::address(),
            Action::ProcessEthereumMessages(..) => Bridge::address(),
            Action::ProcessPolygonMessages(..) => Bridge::address(),
            Action::Null => return None, // action => return Err(anyhow!("{:?} invalid action", action.clone())),
        })
    }

    fn data(&self) -> Vec<u8> {
        encode_action(&self.0.action)
    }

    fn value(&self) -> Vec<u8> {
        match &self.0.action {
            Action::Pay(_recipient, amount, token) => {
                if *token == CUSDC {
                    (BigUint::from(amount) * BigUint::from(pow(BigUint::from(10u32), 12)))
                        .to_bytes_be()
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }

    pub fn sender(&self) -> anyhow::Result<Address> {
        Ok(self.recover_address().unwrap())
    }

    pub fn run<B: Backend>(&self, db: &mut Db<B>) -> anyhow::Result<u64> {
        self.0.run(db, self.sender()?)
    }
}

impl SignedTransaction {
    fn is_withdrawl(&self) -> bool {
        matches!(self.0.action, Action::CreateWithdrawlRequest(_, _))
    }

    pub fn is_seal(&self) -> bool {
        matches!(self.0.action, Action::Seal(_))
    }

    pub fn recover_address(&self) -> Result<Address> {
        self.1.recover_address(&self.signing_data())
    }
}

impl Run for SignedTransaction {
    fn sender(&self) -> Result<Address> {
        self.sender()
    }

    fn run<B: Backend>(&self, db: &mut Db<B>) -> Result<u64> {
        self.0.action.run(db, self.sender()?)
    }
}

pub async fn dispatch(signed_transaction: SignedTransaction) -> Result<u64, anyhow::Error> {
    let receiver = TRANSACTION_QUEUE.push(signed_transaction).await;
    receiver.await.unwrap()
}

pub async fn run(transaction: SignedTransaction) -> Result<u64> {
    let mut db = aquire_db_write_lock!();
    let result = transaction.run(&mut db);
    if transaction.is_withdrawl() && result.is_ok() {
        let pending_withdrawls = Bridge::get_pending_withdrawls(&mut db);
        process_withdrawl(pending_withdrawls.last().unwrap()).await;
    }
    if result.is_ok() {
        db.commit();
    } else {
        db.revert();
    }
    let transacations_file = TRANSACTIONS_FILE.write().await;
    serde_cbor::to_writer(&*transacations_file, &transaction).unwrap();

    result
}

pub async fn apply(transaction: &SignedTransaction) -> Result<()> {
    let backend = DB.get().unwrap().write().await;
    let store_lock = crate::db::StoreLock { guard: backend };
    let mut db = ellipticoin_types::Db {
        backend: store_lock,
        transaction_state: Default::default(),
    };
    let result = transaction.run(&mut db);
    if result.is_ok() {
        db.commit();
    } else {
        db.revert();
    }

    drop(result);
    Ok(())
}

pub async fn new_start_mining_transaction() -> SignedTransaction {
    sign(Action::StartMining(
        HOST.to_string(),
        hash_onion::peel().await,
        hash_onion::layers_left().await.try_into().unwrap(),
    ))
    .await
}

pub async fn new_seal_transaction() -> SignedTransaction {
    sign(Action::Seal(hash_onion::peel().await)).await
}
