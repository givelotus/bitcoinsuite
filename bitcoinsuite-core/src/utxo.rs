use crate::{OutPoint, Script};

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub script: Script,
    pub value: i64,
}
