pub mod proto {
    //! Protobuf structs for data stored by the keyserver.
    include!(concat!(env!("OUT_DIR"), "/chronik.rs"));
}

use std::fmt::Display;

use bitcoinsuite_core::{Bytes, Sha256d};
use bitcoinsuite_error::{ErrorMeta, Result, WrapErr};
use reqwest::{header::CONTENT_TYPE, StatusCode};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ChronikClient {
    http_url: String,
    _ws_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptType {
    Other,
    P2pk,
    P2pkh,
    P2sh,
    P2trCommitment,
    P2trState,
}

#[derive(Debug, Clone)]
pub struct ScriptEndpoint<'payload, 'client> {
    script_type: ScriptType,
    script_payload: &'payload [u8],
    client: &'client ChronikClient,
}

#[derive(Debug, Error, ErrorMeta, PartialEq)]
pub enum ChronikClientError {
    #[critical()]
    #[error("`url` cannot end with '/', got: {0}")]
    CannotHaveTrailingSlashInUrl(String),

    #[critical()]
    #[error("`url` must start with 'https://' or 'http://', got: {0}")]
    InvalidUrlSchema(String),

    #[critical()]
    #[error("HTTP request error")]
    HttpRequestError,

    #[critical()]
    #[error("Unexpected text message: {0}")]
    UnexpectedWsTextMessage(String),

    #[critical()]
    #[error("Chronik error ({status_code}): {error_msg}")]
    ChronikError {
        status_code: StatusCode,
        error: proto::Error,
        error_msg: String,
    },

    #[critical()]
    #[error("Invalid protobuf: {0}")]
    InvalidProtobuf(String),
}

use self::ChronikClientError::*;

impl ChronikClient {
    pub fn new(url: String) -> Result<Self> {
        if url.ends_with('/') {
            return Err(CannotHaveTrailingSlashInUrl(url).into());
        }
        let ws_url = if url.starts_with("https://") {
            "wss://".to_string() + url.strip_prefix("https://").unwrap()
        } else if url.starts_with("http://") {
            "ws://".to_string() + url.strip_prefix("http://").unwrap()
        } else {
            return Err(InvalidUrlSchema(url).into());
        };
        let ws_url = ws_url + "/ws";
        Ok(ChronikClient {
            http_url: url,
            _ws_url: ws_url,
            client: reqwest::Client::new(),
        })
    }

    pub async fn broadcast_tx(&self, raw_tx: Vec<u8>) -> Result<proto::BroadcastTxResponse> {
        self.broadcast_tx_with_slp_check(raw_tx, false).await
    }

    pub async fn broadcast_tx_with_slp_check(
        &self,
        raw_tx: Vec<u8>,
        skip_slp_check: bool,
    ) -> Result<proto::BroadcastTxResponse> {
        let request = proto::BroadcastTxRequest {
            raw_tx,
            skip_slp_check,
        };
        self._post("/broadcast-tx", &request).await
    }

    pub async fn broadcast_txs(
        &self,
        raw_txs: Vec<Vec<u8>>,
    ) -> Result<proto::BroadcastTxsResponse> {
        self.broadcast_txs_with_slp_check(raw_txs, false).await
    }

    pub async fn broadcast_txs_with_slp_check(
        &self,
        raw_txs: Vec<Vec<u8>>,
        skip_slp_check: bool,
    ) -> Result<proto::BroadcastTxsResponse> {
        let request = proto::BroadcastTxsRequest {
            raw_txs,
            skip_slp_check,
        };
        self._post("/broadcast-txs", &request).await
    }

    pub async fn blockchain_info(&self) -> Result<proto::BlockchainInfo> {
        self._get("/blockchain-info").await
    }

    pub async fn block_by_height(&self, height: i32) -> Result<proto::Block> {
        self._get(&format!("/block/{}", height)).await
    }

    pub async fn block_by_hash(&self, hash: &Sha256d) -> Result<proto::Block> {
        self._get(&format!("/block/{}", hash)).await
    }

    pub async fn blocks(
        &self,
        start_height: i32,
        end_height: i32,
    ) -> Result<Vec<proto::BlockInfo>> {
        let blocks: proto::Blocks = self
            ._get(&format!("/blocks/{}/{}", start_height, end_height))
            .await?;
        Ok(blocks.blocks)
    }

    pub async fn tx(&self, txid: &Sha256d) -> Result<proto::Tx> {
        self._get(&format!("/tx/{}", txid)).await
    }

    pub async fn raw_tx(&self, txid: &Sha256d) -> Result<Bytes> {
        use prost::Message as _;
        let response = self
            .client
            .get(format!("{}/raw-tx/{}", self.http_url, txid))
            .send()
            .await
            .wrap_err(HttpRequestError)?;
        let status_code = response.status();
        if status_code != StatusCode::OK {
            let data = response.bytes().await?;
            let error = proto::Error::decode(data.as_ref())
                .wrap_err_with(|| InvalidProtobuf(hex::encode(&data)))?;
            return Err(ChronikError {
                status_code,
                error_msg: error.msg.clone(),
                error,
            }
            .into());
        }
        let bytes = response.bytes().await.wrap_err(HttpRequestError)?;
        Ok(Bytes::from_bytes(bytes))
    }

    pub async fn token(&self, token_id: &Sha256d) -> Result<proto::Token> {
        self._get(&format!("/token/{}", token_id)).await
    }

    pub async fn validate_utxos(
        &self,
        outpoints: Vec<proto::OutPoint>,
    ) -> Result<Vec<proto::UtxoState>> {
        let request = proto::ValidateUtxoRequest { outpoints };
        let response: proto::ValidateUtxoResponse = self._post("/validate-utxos", &request).await?;
        Ok(response.utxo_states)
    }

    pub fn script<'payload, 'client>(
        &'client self,
        script_type: ScriptType,
        script_payload: &'payload [u8],
    ) -> ScriptEndpoint<'payload, 'client> {
        ScriptEndpoint {
            script_type,
            script_payload,
            client: self,
        }
    }

    async fn _post<MRequest: prost::Message, MResponse: prost::Message + Default>(
        &self,
        url_suffix: &str,
        request: &MRequest,
    ) -> Result<MResponse> {
        let response = self
            .client
            .post(format!("{}{}", self.http_url, url_suffix))
            .header(CONTENT_TYPE, "application/x-protobuf")
            .body(request.encode_to_vec())
            .send()
            .await
            .wrap_err(HttpRequestError)?;
        Self::_handle_response(response).await
    }

    async fn _get<MResponse: prost::Message + Default>(
        &self,
        url_suffix: &str,
    ) -> Result<MResponse> {
        let response = self
            .client
            .get(format!("{}{}", self.http_url, url_suffix))
            .header(CONTENT_TYPE, "application/x-protobuf")
            .send()
            .await
            .wrap_err(HttpRequestError)?;
        Self::_handle_response(response).await
    }

    async fn _handle_response<MResponse: prost::Message + Default>(
        response: reqwest::Response,
    ) -> Result<MResponse> {
        use prost::Message as _;
        let status_code = response.status();
        if status_code != StatusCode::OK {
            let data = response.bytes().await?;
            let error = proto::Error::decode(data.as_ref())
                .wrap_err_with(|| InvalidProtobuf(hex::encode(&data)))?;
            return Err(ChronikError {
                status_code,
                error_msg: error.msg.clone(),
                error,
            }
            .into());
        }
        let bytes = response.bytes().await.wrap_err(HttpRequestError)?;
        let response = MResponse::decode(bytes.as_ref())
            .wrap_err_with(|| InvalidProtobuf(hex::encode(&bytes)))?;
        Ok(response)
    }
}

impl ScriptEndpoint<'_, '_> {
    pub async fn history_with_page_size(
        &self,
        page: usize,
        page_size: usize,
    ) -> Result<proto::TxHistoryPage> {
        self.client
            ._get(&format!(
                "/script/{}/{}/history?page={}&page_size={}",
                self.script_type,
                hex::encode(self.script_payload),
                page,
                page_size,
            ))
            .await
    }

    pub async fn history(&self, page: usize) -> Result<proto::TxHistoryPage> {
        self.client
            ._get(&format!(
                "/script/{}/{}/history?page={}",
                self.script_type,
                hex::encode(self.script_payload),
                page,
            ))
            .await
    }

    pub async fn utxos(&self) -> Result<Vec<proto::ScriptUtxos>> {
        let utxos = self
            .client
            ._get::<proto::Utxos>(&format!(
                "/script/{}/{}/utxos",
                self.script_type,
                hex::encode(self.script_payload),
            ))
            .await?;
        Ok(utxos.script_utxos)
    }
}

impl Display for ScriptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            ScriptType::Other => "other",
            ScriptType::P2pk => "p2pk",
            ScriptType::P2pkh => "p2pkh",
            ScriptType::P2sh => "p2sh",
            ScriptType::P2trCommitment => "p2tr-commitment",
            ScriptType::P2trState => "p2tr-state",
        };
        write!(f, "{}", text)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bitcoinsuite_error::Result;

    use crate::{ChronikClient, ChronikClientError};

    #[test]
    fn test_constructor_trailing_slash() -> Result<()> {
        let url = "https://chronik.be.cash/xec/".to_string();
        let err = ChronikClient::new(url.clone())
            .unwrap_err()
            .downcast::<ChronikClientError>()?;
        assert_eq!(err, ChronikClientError::CannotHaveTrailingSlashInUrl(url));
        Ok(())
    }

    #[test]
    fn test_constructor_invalid_schema() -> Result<()> {
        let url = "soap://chronik.be.cash/xec".to_string();
        let err = ChronikClient::new(url.clone())
            .unwrap_err()
            .downcast::<ChronikClientError>()?;
        assert_eq!(err, ChronikClientError::InvalidUrlSchema(url));
        Ok(())
    }
}
