use crate::{
    charge,
    contract::{self, Contract},
    token::Token,
};
use anyhow::{anyhow, Result};
use ellipticoin_macros::db_accessors;
use ellipticoin_types::{
    db::{Backend, Db},
    Address,
};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum PolygonMessage {
    Deposit(u64, Address, Address),
    ProcessWithdrawl(u64, [u8; 32]),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum EthereumMessage {
    SetUSDExchangeRate(BigUint),
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct PendingWithdrawl {
    pub id: u64,
    pub to: Address,
    pub token: Address,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CompletedWithdrawl {
    pub to: Address,
    pub token: Address,
    pub amount: u64,
    pub transaction_hash: [u8; 32],
}

pub struct Bridge;

impl Contract for Bridge {
    const NAME: contract::Name = contract::Name::Bridge;
}

db_accessors!(Bridge {
    ethereum_block_number() -> u64;
    polygon_block_number() -> u64;
    withdrawl_id_counter() -> u64;
    pending_withdrawls() -> Vec<PendingWithdrawl>;
    completed_withdrawl(withdrawl_id: u64) -> CompletedWithdrawl;
});

impl Bridge {
    pub fn process_polygon_messages<B: Backend>(
        db: &mut Db<B>,
        messages: Vec<PolygonMessage>,
        block_number: u64,
    ) -> Result<()> {
        for message in messages {
            match message {
                PolygonMessage::Deposit(amount, token, address) => {
                    Token::mint(db, amount, token, address)
                }
                PolygonMessage::ProcessWithdrawl(withdrawl_id, transaction_hash) => {
                    let mut pending_withdrawls = Self::get_pending_withdrawls(db);
                    let index = pending_withdrawls
                        .iter()
                        .cloned()
                        .position(|pending_withdrawl| pending_withdrawl.id == withdrawl_id)
                        .ok_or(anyhow!("Withdrawl request {} not found", withdrawl_id))?;
                    Self::set_completed_withdrawl(
                        db,
                        withdrawl_id,
                        CompletedWithdrawl {
                            amount: pending_withdrawls[index].amount,
                            to: pending_withdrawls[index].to,
                            token: pending_withdrawls[index].token,
                            transaction_hash,
                        },
                    );
                    pending_withdrawls.remove(index);
                    Self::set_pending_withdrawls(db, pending_withdrawls);
                }
            }
        }
        Self::set_polygon_block_number(db, block_number);
        Ok(())
    }

    pub fn process_ethereum_messages<B: Backend>(
        db: &mut Db<B>,
        messages: Vec<EthereumMessage>,
        block_number: u64,
    ) -> Result<()> {
        for message in messages {
            match message {
                EthereumMessage::SetUSDExchangeRate(usd_exchange_rate) => {
                    Token::set_usd_exchange_rate(db, usd_exchange_rate)
                }
            }
        }
        Self::set_ethereum_block_number(db, block_number.try_into().unwrap());
        Ok(())
    }

    pub fn create_withdrawl_request<B: Backend>(
        db: &mut Db<B>,
        to: Address,
        amount: u64,
        token: Address,
    ) -> Result<()> {
        charge!(db, to, token, amount)?;
        let mut pending_withdrawls = Self::get_pending_withdrawls(db);
        pending_withdrawls.push(PendingWithdrawl {
            id: Self::get_withdrawl_id_counter(db),
            to,
            amount,
            token,
        });
        Self::increment_withdrawl_id_counter(db);
        Self::set_pending_withdrawls(db, pending_withdrawls);

        Ok(())
    }

    fn increment_withdrawl_id_counter<B: Backend>(db: &mut Db<B>) -> u64 {
        let withdrawl_id_counter = Self::get_withdrawl_id_counter(db) + 1;
        Self::set_withdrawl_id_counter(db, withdrawl_id_counter);
        withdrawl_id_counter
    }
}

#[cfg(test)]
mod tests {
    use super::{Bridge, PolygonMessage};
    use crate::{constants::BASE_FACTOR, Token};
    use ellipticoin_test_framework::{
        constants::{actors::ALICE, tokens::APPLES},
        new_db,
    };

    #[test]
    fn test_deposit() {
        let mut db = new_db();
        Bridge::process_polygon_messages(
            &mut db,
            vec![PolygonMessage::Deposit(1 * BASE_FACTOR, APPLES, ALICE)],
            1,
        )
        .unwrap();
        assert_eq!(Token::get_balance(&mut db, ALICE, APPLES,), 1 * BASE_FACTOR);
    }

    #[test]
    fn test_withdrawl() {
        let mut db = new_db();
        Bridge::process_polygon_messages(
            &mut db,
            vec![PolygonMessage::Deposit(1 * BASE_FACTOR, APPLES, ALICE)],
            1,
        )
        .unwrap();
        Bridge::create_withdrawl_request(&mut db, ALICE, 1 * BASE_FACTOR, APPLES).unwrap();
        Bridge::process_polygon_messages(
            &mut db,
            vec![PolygonMessage::ProcessWithdrawl(0, [0; 32])],
            1,
        )
        .unwrap();
        assert_eq!(Token::get_balance(&mut db, ALICE, APPLES), 0);
    }
}
