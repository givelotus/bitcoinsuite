use std::{fmt::Display, str::FromStr};

use bitcoinsuite_core::{Network, Script};
use serde::{Deserialize, Serialize};

use crate::{BitcoinSuiteSlpError, TokenId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValueCoin {
    pub network: Network,
    pub protocol: CoinProtocol,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoinProtocol {
    Satoshis,
    Slp(TokenId),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum CoinProtocolType {
    Satoshis,
    Slp,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValueOutput {
    pub value: u64,
    pub script: Script,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValueOutputs {
    pub coin: ValueCoin,
    pub outputs: Vec<ValueOutput>,
}

impl ValueCoin {
    pub fn bitcoin_uri_prefix(&self) -> Option<&'static str> {
        match (self.network, &self.protocol) {
            (Network::BCH, CoinProtocol::Satoshis) => Some("bitcoincash"),
            (Network::BCH, CoinProtocol::Slp(_)) => Some("simpleledger"),
            (Network::XEC, CoinProtocol::Satoshis) => Some("ecash"),
            (Network::XEC, CoinProtocol::Slp(_)) => Some("etoken"),
            (Network::XPI, _) => Some("payto"),
            (Network::XRG, CoinProtocol::Satoshis) => Some("ergon"),
            (Network::XRG, CoinProtocol::Slp(_)) => None,
        }
    }

    pub fn bip70_content_type_prefix(&self) -> Option<&'static str> {
        match (self.network, &self.protocol) {
            (Network::BCH | Network::XEC | Network::XRG, _) => self.bitcoin_uri_prefix(),
            (Network::XPI, CoinProtocol::Satoshis) => Some("lotus"),
            (Network::XPI, CoinProtocol::Slp(_)) => Some("lotus-slp"),
        }
    }
}

impl Display for ValueOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} to {}", self.value, self.script.hex())
    }
}

impl Display for CoinProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{self:?}").to_ascii_lowercase())
    }
}

impl FromStr for CoinProtocolType {
    type Err = BitcoinSuiteSlpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "satoshis" => Ok(CoinProtocolType::Satoshis),
            "slp" => Ok(CoinProtocolType::Slp),
            _ => Err(BitcoinSuiteSlpError::UnknownCoinProtocol(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BitcoinSuiteSlpError, CoinProtocolType};

    #[test]
    fn test_display() {
        assert_eq!(CoinProtocolType::Satoshis.to_string(), "satoshis");
        assert_eq!(CoinProtocolType::Slp.to_string(), "slp");
    }

    #[test]
    fn test_parse() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            "satoshis".parse::<CoinProtocolType>()?,
            CoinProtocolType::Satoshis
        );
        assert_eq!("slp".parse::<CoinProtocolType>()?, CoinProtocolType::Slp);
        match "Satoshis".parse::<CoinProtocolType>() {
            Err(BitcoinSuiteSlpError::UnknownCoinProtocol(s)) => assert_eq!(s, "Satoshis"),
            _ => panic!("Unexpected parse result"),
        }
        Ok(())
    }
}
