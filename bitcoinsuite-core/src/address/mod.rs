mod cashaddress;
mod lotusaddress;

pub use crate::address::cashaddress::*;
pub use crate::address::lotusaddress::*;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum AddressType {
    P2PKH = 0,
    P2SH = 8,
}
