use std::sync::{atomic, Arc};

use bitcoinsuite_error::{Result, WrapErr};
use serde::{Deserialize, Serialize};

use crate::BitcoindError;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct BitcoindRpcClientConf {
    pub url: String,
    pub rpc_user: String,
    pub rpc_pass: String,
}

#[derive(Debug, Clone)]
pub struct BitcoindRpcClient {
    conf: BitcoindRpcClientConf,
    client: reqwest::Client,
    last_id: Arc<atomic::AtomicUsize>,
}

impl BitcoindRpcClient {
    pub fn new(conf: BitcoindRpcClientConf) -> Self {
        BitcoindRpcClient {
            conf,
            client: reqwest::Client::new(),
            last_id: Arc::new(atomic::AtomicUsize::new(1)),
        }
    }

    pub async fn cmd_text(&self, cmd: &str, args: &[json::JsonValue]) -> Result<String> {
        Ok(self.cmd_json(cmd, args).await?.to_string())
    }

    pub async fn cmd_json(&self, cmd: &str, args: &[json::JsonValue]) -> Result<json::JsonValue> {
        let response = self.cmd_response(cmd, args).await?;
        Self::cmd_handle_error(response).await
    }

    pub async fn test_mempool_accept(
        &self,
        raw_tx: &[u8],
    ) -> Result<std::result::Result<(), String>> {
        let result = self
            .cmd_json("testmempoolaccept", &[json::array![hex::encode(raw_tx)]])
            .await;
        match result {
            Ok(json_result) => {
                let tx_result = &json_result[0];
                if !tx_result["allowed"].as_bool().expect("No 'allowed' field") {
                    return Ok(Err(tx_result["reject-reason"]
                        .as_str()
                        .expect("No 'reject-reason' field")
                        .to_string()));
                }
                Ok(Ok(()))
            }
            Err(report) => match report.downcast::<BitcoindError>()? {
                BitcoindError::JsonRpcCode { message, .. } => Ok(Err(message)),
                bitcoind_error => Err(bitcoind_error.into()),
            },
        }
    }

    pub(crate) async fn cmd_response(
        &self,
        cmd: &str,
        args: &[json::JsonValue],
    ) -> Result<reqwest::Response> {
        self.client
            .post(&self.conf.url)
            .basic_auth(&self.conf.rpc_user, Some(&self.conf.rpc_pass))
            .header(reqwest::header::CONTENT_TYPE, "text/plain")
            .body(
                json::object! {
                    jsonrpc: "1.0",
                    method: cmd,
                    id: self.last_id.fetch_add(1, atomic::Ordering::SeqCst),
                    params: args,
                }
                .to_string(),
            )
            .send()
            .await
            .wrap_err(BitcoindError::Client)
    }

    pub(crate) async fn cmd_handle_error(response: reqwest::Response) -> Result<json::JsonValue> {
        let is_success = response.status().is_success();
        let response_str = response.text().await.wrap_err(BitcoindError::UTF8)?;
        let mut response_json = json::parse(&response_str).wrap_err(BitcoindError::JsonError)?;
        if !is_success {
            let error = &response_json["error"];
            return Err(BitcoindError::JsonRpcCode {
                code: error["code"].as_i32().expect("Unexpected JSON"),
                message: error["message"]
                    .as_str()
                    .expect("Unexpected JSON")
                    .to_string(),
            }
            .into());
        }
        Ok(response_json["result"].take())
    }
}

#[cfg(test)]
mod tests {
    use bitcoinsuite_error::Result;
    use bitcoinsuite_test_utils::bin_folder;

    use crate::{
        instance::{BitcoindChain, BitcoindConf, BitcoindInstance},
        BitcoindError,
    };

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_bitcoind_client() -> Result<()> {
        let conf = BitcoindConf::from_chain_regtest(bin_folder(), BitcoindChain::XEC, vec![])?;
        let mut instance = BitcoindInstance::setup(conf)?;
        instance.wait_for_ready()?;
        let client = instance.rpc_client();
        {
            let result = client.cmd_json("cmddoesntexist", &[]).await;
            let err = result.unwrap_err();
            let err = err.downcast::<BitcoindError>()?;
            assert_eq!(
                err,
                BitcoindError::JsonRpcCode {
                    code: -32601,
                    message: "Method not found".to_string(),
                },
            );
        }
        {
            let block_height_json = client.cmd_json("getblockcount", &[]).await?;
            assert_eq!(block_height_json, 0i32);
        }
        {
            let block_height_text = client.cmd_text("getblockcount", &[]).await?;
            assert_eq!(block_height_text, "0");
        }
        Ok(())
    }
}
