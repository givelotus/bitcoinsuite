use std::path::Path;

use async_trait::async_trait;
use bitcoinsuite_core::{Hashed, Sha256d, UnhashedTx};
use bitcoinsuite_slp::{SlpInterface, SlpNodeInterface, SlpSend, TokenId};
use tonic::transport::Channel;

use crate::{
    bchd_grpc::{
        bchrpc_client::BchrpcClient, get_slp_parsed_script_response::SlpMetadata,
        GetSlpParsedScriptRequest, SlpAction, SubmitTransactionRequest,
    },
    connect_bchd, BchdError,
};

#[derive(Debug, Clone)]
pub struct BchdSlpInterface {
    client: BchrpcClient<Channel>,
}

#[async_trait]
impl SlpInterface for BchdSlpInterface {
    async fn parse_slp_send(&self, tx: &UnhashedTx) -> anyhow::Result<SlpSend> {
        let mut bchd = self.client.clone();
        let output = tx
            .outputs
            .first()
            .ok_or_else(|| anyhow::anyhow!("No outputs"))?;
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
            _ => anyhow::bail!(
                "Not a valid SLP SEND tx ({:?}): {}",
                parsed_script.slp_action(),
                parsed_script.parsing_error
            ),
        };
        Ok(SlpSend {
            token_id: TokenId::from_slice_be(&parsed_script.token_id)?,
            amounts: slp_send.amounts,
        })
    }
}

#[async_trait]
impl SlpNodeInterface for BchdSlpInterface {
    async fn submit_tx(&self, raw_tx: Vec<u8>) -> anyhow::Result<Sha256d> {
        let mut bchd = self.client.clone();
        let response = bchd
            .submit_transaction(SubmitTransactionRequest {
                transaction: raw_tx,
                ..Default::default()
            })
            .await?
            .into_inner();
        Ok(Sha256d::from_slice(&response.hash)?)
    }
}

impl BchdSlpInterface {
    pub fn new(bchd: BchrpcClient<Channel>) -> Self {
        BchdSlpInterface { client: bchd }
    }

    pub async fn connect(url: String, cert_path: impl AsRef<Path>) -> Result<Self, BchdError> {
        let client = connect_bchd(url, cert_path).await?;
        Ok(BchdSlpInterface { client })
    }
}
