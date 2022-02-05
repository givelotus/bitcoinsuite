use std::fmt::Display;

use bitcoinsuite_core::{Network, Script};

use crate::TokenId;

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
