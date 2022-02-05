use std::{collections::HashMap, pin::Pin};

use async_trait::async_trait;
use bitcoinsuite_core::{CashAddress, Sha256d, UnhashedTx};
use bitcoinsuite_error::Result;
use futures::Stream;

use crate::{SlpTx, SlpUtxo, TokenId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlpSend {
    pub token_id: TokenId,
    pub amounts: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenMetadata {
    pub decimals: u32,
}

#[async_trait]
pub trait SlpNodeInterface: Send + Sync {
    async fn submit_tx(&self, raw_tx: Vec<u8>) -> Result<Sha256d>;

    async fn get_token_metadata(
        &self,
        token_ids: &[TokenId],
    ) -> Result<HashMap<TokenId, TokenMetadata>>;

    async fn address_tx_stream(
        &self,
        address: &CashAddress,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<SlpTx>> + Send>>>;

    async fn address_utxos(&self, address: &CashAddress) -> Result<Vec<SlpUtxo>>;
}

#[async_trait]
pub trait SlpInterface: Send + Sync {
    async fn parse_slp_send(&self, tx: &UnhashedTx) -> Result<SlpSend>;
}
