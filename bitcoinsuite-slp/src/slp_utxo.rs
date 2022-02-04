use bitcoinsuite_core::Utxo;

use crate::{SlpToken, TokenId};

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct SlpUtxo {
    pub utxo: Utxo,
    pub token: SlpToken,
    pub token_id: Option<TokenId>,
}
