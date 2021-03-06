pub mod macros;
pub mod tokens;

use crate::token::tokens::TOKEN_METADATA;
use crate::{
    constants::{BASE_FACTOR, BASE_TOKEN_MANTISSA, EXCHANGE_RATE_MANTISSA},
    contract::{self, Contract},
    token::tokens::USD,
    AMM,
};
use anyhow::{anyhow, bail, Result};
use ellipticoin_macros::db_accessors;
use ellipticoin_types::{
    db::{Backend, Db},
    Address,
};
use num_bigint::BigUint;
use num_traits::{cast::ToPrimitive, pow};

use hex;

pub struct Token;

impl Contract for Token {
    const NAME: contract::Name = contract::Name::Token;
}

db_accessors!(Token {
    balance(address: Address, token: Address) -> u64;
    usd_exchange_rate() -> BigUint;
    total_supply(token: Address) -> u64;
});

impl Token {
    pub fn get_underlying_exchange_rate<B: Backend>(db: &mut Db<B>, token: Address) -> u64 {
        Self::underlying_to_amount(db, BASE_FACTOR, token)
    }

    pub fn get_underlying_balance<B: Backend>(
        db: &mut Db<B>,
        address: Address,
        token: Address,
    ) -> u64 {
        let balance = Self::get_balance(db, address, token);
        Self::amount_to_underlying(db, balance, token)
    }

    pub fn get_underlying_total_supply<B: Backend>(db: &mut Db<B>, token: Address) -> u64 {
        let balance = Self::get_total_supply(db, token);
        Self::amount_to_underlying(db, balance, token)
    }

    pub fn amount_to_underlying<B: Backend>(db: &mut Db<B>, amount: u64, token: Address) -> u64 {
        if token == USD {
            let usd_exchange_rate = Token::get_usd_exchange_rate(db);
            (usd_exchange_rate * amount
                / pow(
                    BigUint::from(10u32),
                    BASE_TOKEN_MANTISSA + EXCHANGE_RATE_MANTISSA,
                ))
            .to_u64()
            .unwrap()
        } else {
            amount
        }
    }

    pub fn underlying_to_amount<B: Backend>(
        db: &mut Db<B>,
        underlying_amount: u64,
        token: Address,
    ) -> u64 {
        if token == USD {
            let usd_exchange_rate = Token::get_usd_exchange_rate(db);
            (pow(
                BigUint::from(10u32),
                BASE_TOKEN_MANTISSA + EXCHANGE_RATE_MANTISSA,
            ) * underlying_amount
                / usd_exchange_rate)
                .to_u64()
                .unwrap()
        } else {
            underlying_amount
        }
    }

    pub fn get_price<B: Backend>(db: &mut Db<B>, token: Address) -> u64 {
        if token == USD {
            BASE_FACTOR
        } else {
            let token_supply = AMM::get_pool_supply_of_token(db, token.clone().into());
            let usd_supply = AMM::get_pool_supply_of_usd(db, token.clone().into());
            let price = if token_supply == 0 {
                0
            } else {
                usd_supply * BASE_FACTOR / token_supply
            };
            Self::amount_to_underlying(db, price, USD)
        }
    }

    pub fn transfer<B: Backend>(
        db: &mut Db<B>,
        sender: Address,
        recipient: Address,
        underlying_amount: u64,
        token: Address,
    ) -> Result<()> {
        Self::debit(db, underlying_amount, token, sender)?;
        Self::credit(db, underlying_amount, token, recipient);
        Ok(())
    }

    pub fn mint<B: Backend>(db: &mut Db<B>, amount: u64, token: Address, address: Address) {
        Self::credit(db, amount, token, address);
        let total_supply = Self::get_total_supply(db, token);
        Self::set_total_supply(db, token, total_supply + amount);
    }

    pub fn burn<B: Backend>(
        db: &mut Db<B>,
        amount: u64,
        token: Address,
        address: Address,
    ) -> Result<()> {
        Self::debit(db, amount, token, address)?;
        let total_supply = Self::get_total_supply(db, token);
        Self::set_total_supply(db, token, total_supply - amount);
        Ok(())
    }

    pub fn credit<B: Backend>(db: &mut Db<B>, amount: u64, token: Address, address: Address) {
        let balance = Self::get_balance(db, address, token);
        Self::set_balance(db, address, token, balance + amount)
    }

    pub fn debit<B: Backend>(
        db: &mut Db<B>,
        amount: u64,
        token: Address,
        address: Address,
    ) -> Result<()> {
        let balance = Self::get_balance(db, address, token);

        if amount <= balance {
            Ok(Self::set_balance(db, address, token, balance - amount))
        } else {
            bail!(
                "{} has insufficient balance of {} have {} need {}",
                hex::encode(address),
                TOKEN_METADATA
                    .get(&token)
                    .ok_or(anyhow!("Unknown token"))?
                    .symbol,
                balance as f64 / 1000000.0,
                amount as f64 / 1000000.0
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Token;
    use ellipticoin_test_framework::{
        constants::{
            actors::{ALICE, BOB},
            tokens::APPLES,
        },
        new_db,
    };

    #[test]
    fn test_transfer() {
        let mut db = new_db();
        Token::set_balance(&mut db, ALICE, APPLES, 100);
        Token::transfer(&mut db, ALICE, BOB, 20, APPLES).unwrap();
        assert_eq!(Token::get_balance(&mut db, ALICE, APPLES), 80);
        assert_eq!(Token::get_balance(&mut db, BOB, APPLES), 20);
    }
}
