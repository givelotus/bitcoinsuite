mod cashaddress;

pub use crate::address::cashaddress::*;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum AddressType {
    P2PKH = 0,
    P2SH = 8,
}
