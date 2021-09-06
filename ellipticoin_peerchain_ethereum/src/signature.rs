use crate::{crypto::keccak256, helpers::left_pad};
use anyhow::{anyhow, Result};
use ellipticoin_types::Address;
use k256::{
    ecdsa::{recoverable, VerifyingKey},
    elliptic_curve::sec1::ToEncodedPoint,
};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Signature {
    pub v: [u8; 1],
    pub r: [u8; 32],
    pub s: [u8; 32],
}

impl Signature {
    pub fn recover_address(&self, message: &[u8]) -> Result<Address> {
        let signature = recoverable::Signature::try_from(
            &[
                self.r.to_vec(),
                self.s.to_vec(),
                vec![normalize_recovery_id(self.v[0].into())],
            ]
            .concat()[..],
        )
        .or(Err(anyhow!("error")))?;
        let recovered_key = signature
            .recover_verify_key(&message)
            .or(Err(anyhow!("error")))?;
        Ok(eth_address(&recovered_key))
    }

    pub fn from_v_r_s(v: &[u8], r: &[u8], s: &[u8]) -> Result<Self> {
        Ok(Self {
            v: v[..].try_into().or(Err(anyhow!("error")))?,
            r: left_pad(r, 32).try_into().or(Err(anyhow!("error")))?,
            s: left_pad(s, 32).try_into().or(Err(anyhow!("error")))?,
        })
    }
}

// Copied from https://github.com/gakonst/ethers-rs/blob/4c8d3c81e734c1760443b42a6c2229b68cfe9b3e/ethers-core/src/types/signature.rs#L142 ¯\_(ツ)_/¯
// Also see: https://eips.ethereum.org/EIPS/eip-155
fn normalize_recovery_id(v: u64) -> u8 {
    match v {
        0 => 0,
        1 => 1,
        27 => 0,
        28 => 1,
        v if v >= 35 => ((v - 1) % 2) as _,
        _ => 4,
    }
}

pub fn eth_address(verify_key: &VerifyingKey) -> Address {
    Address(
        keccak256(&verify_key.to_encoded_point(false).to_bytes()[1..])[12..]
            .try_into()
            .unwrap(),
    )
}
