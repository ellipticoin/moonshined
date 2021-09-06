use crate::{
    bridge::{EthereumMessage, PolygonMessage},
    constants::{TOKENS, USD},
    contract::{self, Contract},
    crypto::ed25519_verify,
    Bridge, Ellipticoin, Token, AMM,
};
use anyhow::{anyhow, Result};
use ellipticoin_macros::db_accessors;
use ellipticoin_types::{
    db::{Backend, Db},
    Address, Uint,
};
use serde::{Deserialize, Serialize};

use std::convert::TryInto;

pub struct System;

impl Contract for System {
    const NAME: contract::Name = contract::Name::System;
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub start_transaction_id: u64,
    pub transaction_count: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub transaction_number: u64,
    pub action: Action,
}

impl Transaction {
    pub fn run<B: Backend>(&self, db: &mut Db<B>, sender: Address) -> Result<u64> {
        self.action.run(db, sender)
    }
}

impl Action {
    pub fn run<B: Backend>(&self, db: &mut Db<B>, sender: Address) -> Result<u64> {
        let result = match &self {
            Action::AddLiquidity(amount, token) => {
                AMM::add_liquidity(db, sender, (*amount).try_into()?, *token)
            }
            Action::Buy(underlying_input_amount, input_token, minimum_underlying_output_amount) => {
                let input_amount =
                    Token::underlying_to_amount(db, (*underlying_input_amount).try_into()?, USD);
                let minimum_output_amount = Token::underlying_to_amount(
                    db,
                    (*minimum_underlying_output_amount).try_into()?,
                    *input_token,
                );

                AMM::buy(
                    db,
                    sender,
                    input_amount,
                    *input_token,
                    minimum_output_amount,
                )
            }

            Action::CreatePool(amount, token, underlying_starting_price) => {
                let starting_price =
                    Token::underlying_to_amount(db, (*underlying_starting_price).try_into()?, USD);
                AMM::create_pool(db, sender, (*amount).try_into()?, *token, starting_price)
            }
            Action::CreateWithdrawlRequest(underlying_amount, token) => {
                println!("{:?}", underlying_amount);
                let amount =
                    Token::underlying_to_amount(db, (*underlying_amount).try_into()?, *token);
                println!("{}", amount);
                Bridge::create_withdrawl_request(db, sender, (amount).try_into()?, *token)
            }
            Action::Null => Ok(()),
            Action::Pay(recipient, underlying_amount, token) => {
                let amount =
                    Token::underlying_to_amount(db, (*underlying_amount).try_into()?, *token);
                Token::transfer(db, sender, *recipient, amount, *token)
            }
            Action::RemoveLiquidity(percentage, token) => {
                AMM::remove_liquidity(db, sender, (*percentage).try_into()?, *token)
            }
            Action::Seal(onion_skin) => Ellipticoin::seal(db, sender, *onion_skin),
            Action::Sell(
                underlying_input_amount,
                input_token,
                minimum_underlying_output_amount,
            ) => {
                let input_amount = Token::underlying_to_amount(
                    db,
                    (*underlying_input_amount).try_into()?,
                    *input_token,
                );
                let minimum_output_amount = Token::underlying_to_amount(
                    db,
                    (*minimum_underlying_output_amount).try_into()?,
                    USD,
                );

                AMM::sell(
                    db,
                    sender,
                    input_amount,
                    *input_token,
                    minimum_output_amount,
                )
            }
            Action::ProcessEthereumMessages(messages, block_number) => {
                Bridge::process_ethereum_messages(
                    db,
                    messages.to_vec(),
                    (*block_number).try_into()?,
                )
            }
            Action::ProcessPolygonMessages(messages, block_number) => {
                Bridge::process_polygon_messages(db, messages.to_vec(), (*block_number).try_into()?)
            }
            Action::StartMining(host, onion_skin, layer_count) => Ellipticoin::start_mining(
                db,
                sender,
                host.to_string(),
                *onion_skin,
                (*layer_count).try_into()?,
            ),
        };
        if result.is_ok() {
            System::increment_transaction_number(db, sender);
            let transaction_id = System::increment_transaction_id(db);
            db.commit();
            Ok(transaction_id)
        } else {
            db.revert();
            Err(result.err().unwrap())
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Action {
    AddLiquidity(Uint, Address),
    Buy(Uint, Address, Uint),
    CreatePool(Uint, Address, Uint),
    CreateWithdrawlRequest(Uint, Address),
    ProcessEthereumMessages(Vec<EthereumMessage>, Uint),
    ProcessPolygonMessages(Vec<PolygonMessage>, Uint),
    Null,
    Pay(Address, Uint, Address),
    Sell(Uint, Address, Uint),
    RemoveLiquidity(Uint, Address),
    Seal([u8; 32]),
    StartMining(String, [u8; 32], Uint),
}

impl Default for Action {
    fn default() -> Self {
        Action::Null
    }
}

db_accessors!(System {
    block_number() -> u64;
    blocks() -> Vec<Block>;
    transaction_number(address: Address) -> u64;
    transaction_id_counter() -> u64;
});

impl System {
    pub fn run<B: Backend>(
        db: &mut Db<B>,
        transaction: Transaction,
        sender: Address,
    ) -> Result<u64, anyhow::Error> {
        if Self::get_transaction_number(db, sender) + 1 != transaction.transaction_number {
            return Err(anyhow!(
                "Expected transaction number {} but got {}",
                Self::get_transaction_number(db, sender) + 1,
                transaction.transaction_number
            ));
        }
        let result = transaction.action.run(db, sender);
        if result.is_ok() {
            System::increment_transaction_number(db, sender);
            let transaction_id = System::increment_transaction_id(db);
            db.commit();
            Ok(transaction_id)
        } else {
            db.revert();
            Err(result.err().unwrap())
        }
    }

    pub fn get_next_transaction_number<B: Backend>(db: &mut Db<B>, address: Address) -> u64 {
        if Self::get_transaction_number(db, address) == 0 {
            1
        } else {
            Self::get_transaction_number(db, address) + 1
        }
    }

    pub fn seal_block<B: Backend>(db: &mut Db<B>) {
        System::increment_block_number(db);
        let mut blocks = Self::get_blocks(db);
        let block = if let Some(last_block) = blocks.last() {
            let current_transaction_id = Self::get_transaction_id_counter(db);
            let start_transaction_id =
                last_block.start_transaction_id + last_block.transaction_count;
            Block {
                start_transaction_id,
                transaction_count: current_transaction_id - start_transaction_id,
            }
        } else {
            Default::default()
        };
        // let last_block = blocks.last();
        // let current_transaction_id = Self::get_transaction_id_counter(db);
        blocks.push(block);
        Self::set_blocks(db, blocks)
    }

    pub fn increment_block_number<B: Backend>(db: &mut Db<B>) -> u64 {
        let block_number = Self::get_block_number(db) + 1;
        Self::set_block_number(db, block_number);
        block_number
    }

    pub fn increment_transaction_id<B: Backend>(db: &mut Db<B>) -> u64 {
        let transaction_id = Self::get_transaction_id_counter(db) + 1;
        Self::set_transaction_id_counter(db, transaction_id);
        transaction_id
    }

    pub fn increment_transaction_number<B: Backend>(db: &mut Db<B>, address: Address) {
        let transaction_number = System::get_next_transaction_number(db, address);
        Self::set_transaction_number(db, address, transaction_number);
    }

    pub fn migrate<B: Backend>(
        db: &mut Db<B>,
        sender: Address,
        legacy_address: [u8; 32],
        legacy_signature: Vec<u8>,
    ) -> Result<()> {
        ed25519_verify(sender.as_ref(), &legacy_address, &legacy_signature)?;
        Ellipticoin::harvest(db, Address(legacy_address[..20].try_into().unwrap()))?;
        for token in [TOKENS.to_vec()].concat().iter() {
            let balance = Token::get_balance(
                db,
                Address(legacy_address[..20].try_into().unwrap()),
                *token,
            );
            Token::debit(
                db,
                balance,
                *token,
                Address(legacy_address[..20].try_into().unwrap()),
            )
            .unwrap();
            Token::credit(db, balance, *token, sender);
        }

        for token in TOKENS.iter() {
            let legacy_address: Address = Address(legacy_address[..20].try_into().unwrap());
            if AMM::get_liquidity_providers(db, *token).contains(&legacy_address) {
                let balance = AMM::get_balance(db, legacy_address, *token);
                // println!("{} {} {}", hex::encode(legacy_address), hex::encode(token), balance);
                AMM::transfer(db, legacy_address, sender, balance, *token)?;
                let mut liquidity_providers = AMM::get_liquidity_providers(db, *token);
                liquidity_providers.remove(&legacy_address);
                liquidity_providers.insert(sender);
                AMM::set_liquidity_providers(db, *token, liquidity_providers);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::System;
    use ellipticoin_test_framework::{
        constants::{
            actors::{ALICE, BOB},
            tokens::APPLES,
        },
        new_db,
    };
    use ellipticoin_types::Uint;
    use std::convert::TryFrom;

    #[test]
    fn test_run() {
        let mut db = new_db();
        Token::set_balance(&mut db, ALICE, APPLES, 100);
        let transfer_transaction = Transaction {
            transaction_number: 0,
            action: Action::Pay(BOB, Uint::try_from(20u64).unwrap(), APPLES),
        };
        transfer_transaction.run(&mut db, ALICE).unwrap();
        assert_eq!(Token::get_balance(&mut db, ALICE, APPLES), 80);
        assert_eq!(Token::get_balance(&mut db, BOB, APPLES), 20);
        assert_eq!(System::get_transaction_number(&mut db, ALICE), 1);
    }
}
