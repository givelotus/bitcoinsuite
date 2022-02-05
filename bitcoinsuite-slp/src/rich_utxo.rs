use bitcoinsuite_core::{Network, OutPoint, TxOutput};

use crate::{RichTxBlock, SlpToken, SlpTokenType, SlpTxTypeVariant, TokenId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlpOutput {
    pub token_id: TokenId,
    pub tx_type: SlpTxTypeVariant,
    pub token_type: SlpTokenType,
    pub token: SlpToken,
    pub group_token_id: Option<Box<TokenId>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichUtxo {
    pub outpoint: OutPoint,
    pub block: Option<RichTxBlock>,
    pub is_coinbase: bool,
    pub output: TxOutput,
    pub slp_output: Option<Box<SlpOutput>>,
    pub time_first_seen: i64,
    pub network: Network,
}
