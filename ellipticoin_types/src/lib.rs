pub mod db;
pub mod traits;

pub use db::Db;
use std::ops::{BitXor, Shr};
pub const ADDRESS_LENGTH: usize = 20;
use hex;
use num_bigint::{BigInt, BigUint};
use serde::{Deserialize, Serialize};
use std::{
    array::TryFromSliceError,
    convert::{TryFrom, TryInto},
    fmt::{self, Display, Formatter},
};

#[derive(
    Copy, Serialize, Deserialize, Debug, Default, PartialEq, Clone, Eq, Hash, PartialOrd, Ord,
)]
pub struct Address(pub [u8; ADDRESS_LENGTH]);

#[derive(
    Copy, Serialize, Deserialize, Debug, Default, PartialEq, Clone, Eq, Hash, PartialOrd, Ord,
)]
pub struct Int(i64);

#[derive(
    Copy, Serialize, Deserialize, Debug, Default, PartialEq, Clone, Eq, Hash, PartialOrd, Ord,
)]
pub struct Uint(u64);

#[derive(Debug, Clone)]
pub struct UintOverflow(u64);

#[derive(Debug, Clone)]
pub struct NegativeUintError(i64);

impl fmt::Display for NegativeUintError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "expected positive integer got: {}", self.0)
    }
}

impl fmt::Display for UintOverflow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} is greater than the max uint {}", self.0, i64::MAX)
    }
}

impl Uint {
    pub fn as_i64(&self) -> i64 {
        self.0 as i64
    }
}

impl Uint {
    pub fn to_le_bytes(&self) -> [u8; 8] {
        self.0.to_le_bytes()
    }
}

impl From<Uint> for u64 {
    fn from(n: Uint) -> Self {
        n.0 as u64
    }
}

impl From<Uint> for BigUint {
    fn from(n: Uint) -> Self {
        BigUint::from(n.0 as u64)
    }
}

impl From<&Uint> for BigUint {
    fn from(n: &Uint) -> Self {
        BigUint::from(n.0 as u64)
    }
}

impl From<Uint> for BigInt {
    fn from(n: Uint) -> Self {
        BigInt::from(n.0 as u64)
    }
}

impl From<&Uint> for BigInt {
    fn from(n: &Uint) -> Self {
        BigInt::from(n.0 as u64)
    }
}

impl TryFrom<usize> for Uint {
    type Error = UintOverflow;
    fn try_from(n: usize) -> Result<Self, Self::Error> {
        if n <= i64::MAX as usize {
            Ok(Self(n as u64))
        } else {
            Err(UintOverflow(n as u64))
        }
    }
}

impl TryFrom<u64> for Uint {
    type Error = UintOverflow;
    fn try_from(n: u64) -> Result<Self, Self::Error> {
        if n <= i64::MAX as u64 {
            Ok(Self(n))
        } else {
            Err(UintOverflow(n))
        }
    }
}

impl TryFrom<i64> for Uint {
    type Error = NegativeUintError;
    fn try_from(n: i64) -> Result<Self, Self::Error> {
        if n >= 0 {
            Ok(Self(n.try_into().unwrap()))
        } else {
            Err(NegativeUintError(n))
        }
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

// impl Into<[u8; 20]> for Address {
//     fn  into(self) -> [u8; 20] {
//         self.0
//     }
// }

impl From<&Address> for [u8; 20] {
    fn from(address: &Address) -> Self {
        address.0
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0.as_ref()))
    }
}

impl TryFrom<&str> for Address {
    type Error = hex::FromHexError;
    fn try_from(address: &str) -> Result<Self, Self::Error> {
        let a = hex::decode(address.trim_start_matches("0x"))?;
        Ok(Address(a.try_into().unwrap()))
    }
}

impl TryFrom<&[u8]> for Address {
    type Error = TryFromSliceError;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Ok(Address(bytes.try_into()?))
    }
}

impl BitXor for Address {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(
            self.0
                .iter()
                .zip(rhs.0.iter())
                .map(|(&a, &b)| a ^ b)
                .collect::<Vec<u8>>()
                .try_into()
                .unwrap(),
        )
    }
}
impl Shr<usize> for Address {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        let mut lhs = self.0.clone();
        lhs.rotate_right(rhs);
        Self(lhs)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_negative_uint() {
        assert_eq!(
            Uint::try_from(-20i64).err().unwrap().to_string(),
            "expected positive integer got: -20"
        );
    }

    #[test]
    fn test_uint_overflow() {
        assert_eq!(
            Uint::try_from(i64::MAX as u64 + 1)
                .err()
                .unwrap()
                .to_string(),
            "9223372036854775808 is greater than the max uint 9223372036854775807"
        );
    }
}
