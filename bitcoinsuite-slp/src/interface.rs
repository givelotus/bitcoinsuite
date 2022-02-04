use async_trait::async_trait;
use bitcoinsuite_core::{Sha256d, UnhashedTx};
use bitcoinsuite_error::Result;

use crate::TokenId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlpSend {
    pub token_id: TokenId,
    pub amounts: Vec<u64>,
}

#[async_trait]
pub trait SlpNodeInterface: Send + Sync {
    async fn submit_tx(&self, raw_tx: Vec<u8>) -> Result<Sha256d>;
}

#[async_trait]
pub trait SlpInterface: Send + Sync {
    async fn parse_slp_send(&self, tx: &UnhashedTx) -> Result<SlpSend>;
}
