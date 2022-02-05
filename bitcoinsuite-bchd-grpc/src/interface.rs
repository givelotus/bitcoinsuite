use std::{collections::HashMap, path::Path, pin::Pin, sync::Arc};

use async_trait::async_trait;
use bitcoinsuite_core::{
    CashAddress, Hashed, Net, OutPoint, Script, Sha256d, UnhashedTx, Utxo, BCHREG, BITCOINCASH,
};
use bitcoinsuite_error::{ErrorMeta, Report, Result, WrapErr};
use bitcoinsuite_slp::{
    SlpAmount, SlpInterface, SlpNodeInterface, SlpSend, SlpToken, SlpTx, SlpUtxo, TokenId,
    TokenMetadata,
};
use futures::{Stream, StreamExt};
use thiserror::Error;
use tokio::sync::RwLock;
use tonic::transport::Channel;

use crate::{
    bchd_grpc::{
        self, bchrpc_client::BchrpcClient, get_slp_parsed_script_response::SlpMetadata,
        GetAddressUnspentOutputsRequest, GetSlpParsedScriptRequest, SlpAction,
        SubmitTransactionRequest, SubscribeTransactionsRequest, TransactionFilter,
    },
    connect_bchd, to_slp_tx,
};

#[derive(Debug, Clone)]
pub struct BchdSlpInterface {
    client: BchrpcClient<Channel>,
    token_metadata_cache: Arc<RwLock<HashMap<TokenId, TokenMetadata>>>,
    net: Net,
}

#[derive(Error, ErrorMeta, Debug)]
pub enum BchdSlpError {
    #[invalid_client_input()]
    #[error("No outputs")]
    NoOutputs,

    #[invalid_client_input()]
    #[error("Not a valid SLP SEND tx ({0:?}): {1}")]
    InvalidSlpSend(SlpAction, String),

    #[invalid_client_input()]
    #[error("Invalid tx: {0}")]
    SubmitTxFail(String),

    #[critical()]
    #[error("gRPC fail")]
    GrpcFail,
}

use self::BchdSlpError::*;

#[async_trait]
impl SlpInterface for BchdSlpInterface {
    async fn parse_slp_send(&self, tx: &UnhashedTx) -> Result<SlpSend> {
        let mut bchd = self.client.clone();
        let output = tx.outputs.first().ok_or(NoOutputs)?;
        let parsed_script = bchd
            .get_slp_parsed_script(GetSlpParsedScriptRequest {
                slp_opreturn_script: output.script.bytecode().to_vec(),
            })
            .await?
            .into_inner();
        let slp_send = match parsed_script.slp_metadata {
            Some(SlpMetadata::V1Send(slp_send))
                if parsed_script.slp_action() == SlpAction::SlpV1Send =>
            {
                slp_send
            }
            _ => {
                return Err(
                    InvalidSlpSend(parsed_script.slp_action(), parsed_script.parsing_error).into(),
                )
            }
        };
        Ok(SlpSend {
            token_id: TokenId::from_slice_be(&parsed_script.token_id)?,
            amounts: slp_send.amounts,
        })
    }
}

#[async_trait]
impl SlpNodeInterface for BchdSlpInterface {
    async fn submit_tx(&self, raw_tx: Vec<u8>) -> Result<Sha256d> {
        let mut bchd = self.client.clone();
        let response = bchd
            .submit_transaction(SubmitTransactionRequest {
                transaction: raw_tx,
                ..Default::default()
            })
            .await
            .map_err(|err| Report::msg(err.message().to_string()))?
            .into_inner();
        Ok(Sha256d::from_slice(&response.hash)?)
    }

    async fn get_token_metadata(
        &self,
        token_ids: &[TokenId],
    ) -> Result<HashMap<TokenId, TokenMetadata>> {
        let mut bchd = self.client.clone();
        let mut remaining_token_ids = Vec::new();
        let mut token_metadata_map = HashMap::with_capacity(token_ids.len());
        {
            let cache = self.token_metadata_cache.read().await;
            for token_id in token_ids {
                match cache.get(token_id) {
                    Some(metadata) => {
                        token_metadata_map.insert(token_id.clone(), metadata.clone());
                    }
                    None => remaining_token_ids.push(token_id.clone()),
                }
            }
        }
        if !remaining_token_ids.is_empty() {
            use crate::bchd_grpc::{slp_token_metadata::TypeMetadata, GetSlpTokenMetadataRequest};
            let mut cache = self.token_metadata_cache.write().await;
            let response = bchd
                .get_slp_token_metadata(GetSlpTokenMetadataRequest {
                    token_ids: remaining_token_ids
                        .iter()
                        .map(|token_id| token_id.as_slice_be().to_vec())
                        .collect(),
                })
                .await?
                .into_inner();
            for metadata in response.token_metadata {
                if let Some(TypeMetadata::V1Fungible(type_metadata)) = metadata.type_metadata {
                    let token_metadata = TokenMetadata {
                        decimals: type_metadata.decimals,
                    };
                    let token_id = TokenId::from_slice_be(&metadata.token_id)?;
                    cache.insert(token_id.clone(), token_metadata.clone());
                    token_metadata_map.insert(token_id, token_metadata);
                }
            }
        }
        Ok(token_metadata_map)
    }

    async fn address_tx_stream(
        &self,
        address: &CashAddress,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<SlpTx>> + Send>>> {
        let prefix = match self.net {
            Net::Mainnet => BITCOINCASH,
            Net::Regtest => BCHREG,
        };
        self.tx_stream(TransactionFilter {
            addresses: vec![address.with_prefix(prefix).into_string()],
            ..TransactionFilter::default()
        })
        .await
    }

    async fn address_utxos(&self, address: &CashAddress) -> Result<Vec<SlpUtxo>> {
        let mut bchd = self.client.clone();
        let prefix = match self.net {
            Net::Mainnet => BITCOINCASH,
            Net::Regtest => BCHREG,
        };
        let utxos = bchd
            .get_address_unspent_outputs(GetAddressUnspentOutputsRequest {
                address: address.with_prefix(prefix).into_string(),
                include_mempool: true,
                include_token_metadata: false,
            })
            .await
            .wrap_err(GrpcFail)?
            .into_inner();
        Ok(utxos
            .outputs
            .into_iter()
            .flat_map(|utxo| {
                let outpoint = utxo.outpoint?;
                Some(SlpUtxo {
                    utxo: Utxo {
                        outpoint: OutPoint {
                            txid: Sha256d::from_slice(&outpoint.hash).ok()?,
                            out_idx: outpoint.index,
                        },
                        script: Script::from_slice(&utxo.pubkey_script),
                        value: utxo.value,
                    },
                    token: utxo
                        .slp_token
                        .as_ref()
                        .map(|slp_token| SlpToken {
                            amount: SlpAmount::new(slp_token.amount.into()),
                            is_mint_baton: slp_token.is_mint_baton,
                        })
                        .unwrap_or_default(),
                    token_id: utxo
                        .slp_token
                        .and_then(|slp_token| TokenId::from_slice_be(&slp_token.token_id).ok()),
                })
            })
            .collect())
    }
}

impl BchdSlpInterface {
    pub fn new(bchd: BchrpcClient<Channel>, net: Net) -> Self {
        BchdSlpInterface {
            client: bchd,
            token_metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            net,
        }
    }

    pub async fn connect(url: String, cert_path: impl AsRef<Path>, net: Net) -> Result<Self> {
        let client = connect_bchd(url, cert_path).await?;
        Ok(BchdSlpInterface::new(client, net))
    }

    async fn tx_stream(
        &self,
        tx_filter: TransactionFilter,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<SlpTx>> + Send>>> {
        let mut bchd = self.client.clone();
        let stream = bchd
            .subscribe_transactions(SubscribeTransactionsRequest {
                subscribe: Some(tx_filter),
                unsubscribe: None,
                include_in_block: false,
                include_mempool: true,
                serialize_tx: false,
            })
            .await
            .wrap_err(GrpcFail)?
            .into_inner();
        Ok(Box::pin(stream.map(|tx_notification| -> Result<_> {
            let tx_notification = tx_notification.wrap_err(GrpcFail)?;
            let tx = tx_notification.transaction.expect("No tx in notification");
            match tx {
                bchd_grpc::transaction_notification::Transaction::UnconfirmedTransaction(tx) => {
                    Ok(to_slp_tx(tx.transaction.expect("No mempool tx")))
                }
                _ => unreachable!("Only unconfirmed transactions filtered"),
            }
        })))
    }
}
