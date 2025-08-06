use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::BitcoinSuiteError;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Network {
    BCH,
    XEC,
    XPI,
    XRG,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum Net {
    #[default]
    Mainnet,
    Regtest,
}

impl Network {
    pub fn dust_amount(&self) -> i64 {
        match self {
            Network::XRG => 2,
            _ => 546,
        }
    }

    pub fn coin_decimals(&self) -> u32 {
        match self {
            Network::XEC => 2,
            Network::XPI => 6,
            Network::BCH | Network::XRG => 8,
        }
    }

    pub fn block_spacing(&self) -> u32 {
        match self {
            Network::XEC | Network::BCH | Network::XRG => 600,
            Network::XPI => 120,
        }
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl FromStr for Network {
    type Err = BitcoinSuiteError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BCH" => Ok(Network::BCH),
            "XEC" => Ok(Network::XEC),
            "XPI" => Ok(Network::XPI),
            "XRG" => Ok(Network::XRG),
            _ => Err(BitcoinSuiteError::UnknownNetwork(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BitcoinSuiteError, Network};

    #[test]
    fn test_display() {
        assert_eq!(Network::BCH.to_string(), "BCH");
        assert_eq!(Network::XEC.to_string(), "XEC");
        assert_eq!(Network::XPI.to_string(), "XPI");
        assert_eq!(Network::XRG.to_string(), "XRG");
    }

    #[test]
    fn test_parse() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!("BCH".parse::<Network>()?, Network::BCH);
        assert_eq!("XEC".parse::<Network>()?, Network::XEC);
        assert_eq!("XPI".parse::<Network>()?, Network::XPI);
        assert_eq!("XRG".parse::<Network>()?, Network::XRG);
        match "bch".parse::<Network>() {
            Err(BitcoinSuiteError::UnknownNetwork(s)) => assert_eq!(s, "bch"),
            _ => panic!("Unexpected parse result"),
        }
        Ok(())
    }
}
