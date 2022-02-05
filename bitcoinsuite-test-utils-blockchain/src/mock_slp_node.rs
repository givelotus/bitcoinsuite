use std::{collections::HashMap, pin::Pin, sync::Arc};

use async_trait::async_trait;
use bitcoinsuite_core::{CashAddress, Hashed, Sha256d};
use bitcoinsuite_slp::{SlpNodeInterface, SlpTx, SlpUtxo, TokenId, TokenMetadata};
use futures::{Stream, StreamExt};
use raipay_log::Result;
use tokio::sync::{broadcast, Mutex};
use tokio_stream::wrappers::BroadcastStream;

pub struct MockSlpNode {
    pub utxos: Arc<Mutex<HashMap<CashAddress<'static>, Vec<SlpUtxo>>>>,
    pub address_tx_sender: broadcast::Sender<SlpTx>,
    pub address_tx_receiver: broadcast::Receiver<SlpTx>,
}

impl MockSlpNode {
    pub fn new() -> Self {
        let (address_tx_sender, address_tx_receiver) = broadcast::channel(10);
        MockSlpNode {
            utxos: Arc::new(Mutex::new(HashMap::new())),
            address_tx_sender,
            address_tx_receiver,
        }
    }
}

#[async_trait]
impl SlpNodeInterface for MockSlpNode {
    async fn submit_tx(&self, raw_tx: Vec<u8>) -> Result<Sha256d> {
        Ok(Sha256d::digest(raw_tx.into()))
    }

    async fn get_token_metadata(
        &self,
        _token_ids: &[TokenId],
    ) -> Result<HashMap<TokenId, TokenMetadata>> {
        Ok(HashMap::new())
    }

    async fn address_tx_stream(
        &self,
        _address: &CashAddress,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<SlpTx>> + Send>>> {
        Ok(Box::pin(
            BroadcastStream::new(self.address_tx_sender.subscribe()).map(|tx| Ok(tx?)),
        ))
    }

    async fn address_utxos(&self, address: &CashAddress) -> Result<Vec<SlpUtxo>> {
        Ok(self
            .utxos
            .lock()
            .await
            .get(address)
            .cloned()
            .unwrap_or_default())
    }
}
