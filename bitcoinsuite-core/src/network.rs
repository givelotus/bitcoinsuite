use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Network {
    BCH,
    XEC,
    XPI,
    XRG,
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
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
