use std::str::FromStr;
use thiserror::Error;

use crate::{BytesMut, Hashed, Net, Script, Sha256};

pub const LOTUS_ADDRESS_CHECKSUM_LEN: usize = 4;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LotusAddress {
    prefix: String,
    net: Net,
    lotus_addr: String,
    script: Script,
}

#[derive(Error, Clone, Debug, PartialEq)]
pub enum LotusAddressError {
    #[error("Missing prefix")]
    MissingPrefix,

    #[error("Missing checksum")]
    MissingChecksum,

    #[error("Missing net character")]
    MissingNetChar,

    #[error("Unsupported net {0}")]
    UnsupportedNet(char),

    #[error("Invalid base58")]
    InvalidBase58(bs58::decode::Error),

    #[error("Missing base58")]
    MissingBase58,

    #[error("Missing payload")]
    MissingPayload,

    #[error("Invalid payload type: {0}")]
    InvalidPayloadType(u8),

    #[error("Invalid checksum, expected {expected} but got {actual}")]
    InvalidChecksum { expected: String, actual: String },
}

use self::LotusAddressError::*;

impl LotusAddress {
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn net(&self) -> Net {
        self.net
    }

    pub fn script(&self) -> &Script {
        &self.script
    }

    pub fn as_str(&self) -> &str {
        &self.lotus_addr
    }
}

impl FromStr for LotusAddress {
    type Err = LotusAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // "lotus" part of an address
        let prefix = s
            .chars()
            .take_while(|&c| (c.is_ascii_alphabetic() && c.is_lowercase()) || c.is_ascii_digit())
            .collect::<String>();
        if prefix.is_empty() {
            return Err(MissingPrefix);
        }
        // net: "_" for mainnet, "R" for regtest, "T" for testnet (unsupported)
        let net_char = s.chars().nth(prefix.len()).ok_or(MissingNetChar)?;
        let net = match net_char {
            '_' => Net::Mainnet,
            'R' => Net::Regtest,
            _ => return Err(UnsupportedNet(net_char)),
        };
        // Base58 encoded data
        let data_b58 = &s[prefix.len() + 1..];
        let data = bs58::decode(&data_b58).into_vec().map_err(InvalidBase58)?;
        // First byte indicates payload type. Currently only "0" is supported
        let payload_type = *data.get(0).ok_or(MissingBase58)?;
        if payload_type != 0 {
            return Err(InvalidPayloadType(payload_type));
        }
        // Remainder is the payload, here the scriptPubKey
        let checksum_end_idx = data
            .len()
            .checked_sub(LOTUS_ADDRESS_CHECKSUM_LEN)
            .ok_or(MissingChecksum)?;
        let payload = data.get(1..checksum_end_idx).ok_or(MissingChecksum)?;
        if payload.is_empty() {
            return Err(MissingPayload);
        }
        let expected_checksum = &data[data.len() - LOTUS_ADDRESS_CHECKSUM_LEN..];

        // The data that will be hashed for the checksum
        let mut checksum_preimage = BytesMut::new();
        checksum_preimage.put_slice(prefix.as_bytes());
        checksum_preimage.put_slice(&[net_char as u8, payload_type as u8]);
        checksum_preimage.put_slice(payload);
        let checksum_hash = Sha256::digest(checksum_preimage.freeze());
        let actual_checksum = &checksum_hash.as_slice()[..LOTUS_ADDRESS_CHECKSUM_LEN];

        // Verify checksum
        if expected_checksum != actual_checksum {
            return Err(InvalidChecksum {
                expected: hex::encode(expected_checksum),
                actual: hex::encode(actual_checksum),
            });
        }
        let script = Script::from_slice(payload);
        Ok(LotusAddress {
            prefix,
            net,
            lotus_addr: s.to_string(),
            script,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{Hashed, LotusAddress, LotusAddressError, Net, Script, ShaRmd160};

    #[test]
    fn decode_lotus_address() -> Result<(), Box<dyn std::error::Error>> {
        {
            let address =
                "lotus_16PSJNf1EDEfGvaYzaXJCJZrXH4pgiTo7kyW61iGi".parse::<LotusAddress>()?;
            assert_eq!(address.prefix(), "lotus");
            assert_eq!(address.net(), Net::Mainnet);
            assert_eq!(
                address.script(),
                &Script::p2pkh(&ShaRmd160::from_hex(
                    "b50b86a893d80c9e2ee72b199612374b7b4c1cd8",
                )?),
            );
            assert_eq!(
                address.as_str(),
                "lotus_16PSJNf1EDEfGvaYzaXJCJZrXH4pgiTo7kyW61iGi",
            );
        }
        {
            let address = "lotus_1PrQReKdmXH6hyCk4NFR398HeWxvJWW4E3jjM3".parse::<LotusAddress>()?;
            assert_eq!(address.prefix(), "lotus");
            assert_eq!(address.net(), Net::Mainnet);
            assert_eq!(
                address.script(),
                &Script::p2sh(&ShaRmd160::from_hex(
                    "260617ebf668c9102f71ce24aba97fcaaf9c666a",
                )?),
            );
            assert_eq!(
                address.as_str(),
                "lotus_1PrQReKdmXH6hyCk4NFR398HeWxvJWW4E3jjM3",
            );
        }

        assert_eq!(
            "A".parse::<LotusAddress>().unwrap_err(),
            LotusAddressError::MissingPrefix,
        );
        assert_eq!(
            "lotus".parse::<LotusAddress>().unwrap_err(),
            LotusAddressError::MissingNetChar,
        );
        assert_eq!(
            "lotusP".parse::<LotusAddress>().unwrap_err(),
            LotusAddressError::UnsupportedNet('P'),
        );
        assert_eq!(
            "lotus_".parse::<LotusAddress>().unwrap_err(),
            LotusAddressError::MissingBase58,
        );
        assert_eq!(
            "lotusR".parse::<LotusAddress>().unwrap_err(),
            LotusAddressError::MissingBase58,
        );
        assert_eq!(
            "lotus_0".parse::<LotusAddress>().unwrap_err(),
            LotusAddressError::InvalidBase58(bs58::decode::Error::InvalidCharacter {
                character: '0',
                index: 0
            }),
        );
        assert_eq!(
            "lotus_1".parse::<LotusAddress>().unwrap_err(),
            LotusAddressError::MissingChecksum,
        );
        assert_eq!(
            "lotus_1111".parse::<LotusAddress>().unwrap_err(),
            LotusAddressError::MissingChecksum,
        );
        assert_eq!(
            "lotus_11111".parse::<LotusAddress>().unwrap_err(),
            LotusAddressError::MissingPayload,
        );
        assert_eq!(
            "lotus_111111".parse::<LotusAddress>().unwrap_err(),
            LotusAddressError::InvalidChecksum {
                expected: "00000000".to_string(),
                actual: "66276ef9".to_string(),
            },
        );

        Ok(())
    }
}
