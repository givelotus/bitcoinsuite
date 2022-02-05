use std::sync::Mutex;

use bitcoinsuite_core::Hashed;
use bitcoinsuite_error::{ErrorMeta, Result};
use nng::{Message, Protocol, Socket};
use thiserror::Error;

use crate::{
    field::OptionExt,
    nng_interface_generated::nng_interface::{
        BlockHash, BlockHashArgs, BlockHeight, BlockHeightArgs, BlockIdentifier,
        GetBlockRangeRequest, GetBlockRangeRequestArgs, GetBlockRangeResponse, GetBlockRequest,
        GetBlockRequestArgs, GetBlockResponse, GetBlockSliceRequest, GetBlockSliceRequestArgs,
        GetBlockSliceResponse, GetMempoolRequest, GetMempoolRequestArgs, GetMempoolResponse, Hash,
        RpcCall, RpcCallArgs, RpcRequest, RpcResult,
    },
    structs,
};

pub struct RpcInterface {
    sock: Socket,
    mutex: Mutex<()>,
}

#[derive(Error, Debug, ErrorMeta)]
pub enum RpcInterfaceError {
    #[critical()]
    #[error("RPC error ({error_code}): {message}")]
    RpcError { error_code: i32, message: String },
}

use self::RpcInterfaceError::*;

impl RpcInterface {
    pub fn open(url: &str) -> Result<Self> {
        let sock = Socket::new(Protocol::Req0)?;
        sock.dial(url)?;
        Ok(RpcInterface {
            sock,
            mutex: Mutex::new(()),
        })
    }

    pub fn get_block(&self, block_id: structs::BlockIdentifier) -> Result<structs::Block> {
        let mut fbb = flatbuffers::FlatBufferBuilder::with_capacity(1024);
        let request = match block_id {
            structs::BlockIdentifier::Height(height) => {
                let block_height = BlockHeight::create(&mut fbb, &BlockHeightArgs { height });
                GetBlockRequest::create(
                    &mut fbb,
                    &GetBlockRequestArgs {
                        block_id_type: BlockIdentifier::Height,
                        block_id: Some(block_height.as_union_value()),
                    },
                )
            }
            structs::BlockIdentifier::Hash(blockhash) => {
                let block_hash = BlockHash::create(
                    &mut fbb,
                    &BlockHashArgs {
                        hash: Some(&Hash(blockhash.byte_array().array())),
                    },
                );
                GetBlockRequest::create(
                    &mut fbb,
                    &GetBlockRequestArgs {
                        block_id_type: BlockIdentifier::Hash,
                        block_id: Some(block_hash.as_union_value()),
                    },
                )
            }
        };
        let rpc_call = RpcCall::create(
            &mut fbb,
            &RpcCallArgs {
                rpc_type: RpcRequest::GetBlockRequest,
                rpc: Some(request.as_union_value()),
            },
        );
        fbb.finish(rpc_call, None);
        let msg = self.tranceive(&fbb)?;
        let response = flatbuffers::root::<GetBlockResponse>(self.handle_msg(&msg)?)?;
        let block = response.block().field("GetBlockResponse.block")?;
        structs::Block::from_fbs(block)
    }

    pub fn get_block_range(
        &self,
        start_height: i32,
        num_blocks: u32,
    ) -> Result<Vec<structs::Block>> {
        let mut fbb = flatbuffers::FlatBufferBuilder::with_capacity(1024);
        let request = GetBlockRangeRequest::create(
            &mut fbb,
            &GetBlockRangeRequestArgs {
                start_height,
                num_blocks,
            },
        );
        let rpc_call = RpcCall::create(
            &mut fbb,
            &RpcCallArgs {
                rpc_type: RpcRequest::GetBlockRangeRequest,
                rpc: Some(request.as_union_value()),
            },
        );
        fbb.finish(rpc_call, None);
        let msg = self.tranceive(&fbb)?;
        let response = flatbuffers::root::<GetBlockRangeResponse>(self.handle_msg(&msg)?)?;
        response
            .blocks()
            .field("GetBlockRangeResponse.blocks")?
            .into_iter()
            .map(structs::Block::from_fbs)
            .collect::<Result<Vec<_>>>()
    }

    pub fn get_block_slice(&self, file_num: u32, data_pos: u32, num_bytes: u32) -> Result<Vec<u8>> {
        let mut fbb = flatbuffers::FlatBufferBuilder::with_capacity(1024);
        let request = GetBlockSliceRequest::create(
            &mut fbb,
            &GetBlockSliceRequestArgs {
                file_num,
                data_pos,
                num_bytes,
            },
        );
        let rpc_call = RpcCall::create(
            &mut fbb,
            &RpcCallArgs {
                rpc_type: RpcRequest::GetBlockSliceRequest,
                rpc: Some(request.as_union_value()),
            },
        );
        fbb.finish(rpc_call, None);
        let msg = self.tranceive(&fbb)?;
        let response = flatbuffers::root::<GetBlockSliceResponse>(self.handle_msg(&msg)?)?;
        Ok(response
            .data()
            .field("GetBlockSliceResponse.data")?
            .to_vec())
    }

    pub fn get_mempool(&self) -> Result<Vec<structs::MempoolTx>> {
        let mut fbb = flatbuffers::FlatBufferBuilder::with_capacity(1024);
        let request = GetMempoolRequest::create(&mut fbb, &GetMempoolRequestArgs {});
        let rpc_call = RpcCall::create(
            &mut fbb,
            &RpcCallArgs {
                rpc_type: RpcRequest::GetMempoolRequest,
                rpc: Some(request.as_union_value()),
            },
        );
        fbb.finish(rpc_call, None);
        let msg = self.tranceive(&fbb)?;
        let response = flatbuffers::root::<GetMempoolResponse>(self.handle_msg(&msg)?)?;
        response
            .txs()
            .field("txs")?
            .into_iter()
            .map(structs::MempoolTx::from_fbs)
            .collect::<Result<_>>()
    }

    fn tranceive(&self, fbb: &flatbuffers::FlatBufferBuilder) -> Result<Message> {
        let _guard = self.mutex.lock().expect("Acquire mutex failed");
        self.sock
            .send(fbb.finished_data())
            .map_err(|(_, err)| err)?;
        let resp = self.sock.recv()?;
        Ok(resp)
    }

    fn handle_msg<'a>(&self, msg: &'a Message) -> Result<&'a [u8]> {
        let result = flatbuffers::root::<RpcResult>(&msg[..])?;
        if result.is_success() {
            result.data().field("data")
        } else {
            Err(RpcError {
                error_code: result.error_code(),
                message: result.error_msg().field("error_msg")?.to_string(),
            }
            .into())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, str::FromStr};

    use bitcoinsuite_bitcoind::instance::{BitcoindChain, BitcoindConf, BitcoindInstance};
    use bitcoinsuite_core::{Hashed, Sha256d};
    use bitcoinsuite_error::Result;
    use bitcoinsuite_test_utils::bin_folder;
    use tempdir::TempDir;

    use crate::{BlockIdentifier, RpcInterface};

    #[test]
    fn test_rpc() -> Result<()> {
        bitcoinsuite_error::install()?;
        let ipc_dir = TempDir::new("ipc_rpc_dir")?;
        let rpc_url = format!(
            "ipc://{}",
            ipc_dir.path().join("rpc.pipe").to_string_lossy()
        );
        let rpc_arg = format!("-nngrpc={}", rpc_url);
        let conf = BitcoindConf::from_chain_regtest(
            bin_folder(),
            BitcoindChain::XPI,
            vec![OsString::from_str(&rpc_arg)?],
        )?;
        let mut instance = BitcoindInstance::setup(conf)?;
        instance.wait_for_ready()?;
        let rpc = RpcInterface::open(&rpc_url)?;
        test_get_block(&mut instance, &rpc)?;
        instance.cleanup()?;
        Ok(())
    }

    fn test_get_block(instance: &mut BitcoindInstance, rpc: &RpcInterface) -> Result<()> {
        assert!(rpc.get_mempool()?.is_empty());
        let genesis_block_hash = instance.cmd_string("getblockhash", &["0"])?;
        let genesis_block_header =
            instance.cmd_string("getblockheader", &[&genesis_block_hash, "false"])?;
        let json_block = instance.cmd_json("getblock", &[&genesis_block_hash, "2"])?;
        let block = rpc.get_block(BlockIdentifier::Height(0))?;
        assert_eq!(hex::encode(&block.header.raw), genesis_block_header);
        assert_eq!(block.header.hash.to_hex_be(), genesis_block_hash);
        assert_eq!(block.header.prev_hash.as_slice(), [0; 32]);
        assert_eq!(
            format!("{:08x}", block.header.n_bits),
            json_block["bits"].as_str().unwrap()
        );
        assert_eq!(block.header.timestamp, json_block["time"].as_u64().unwrap());
        assert_eq!(block.metadata, vec![]);
        assert_eq!(block.txs.len(), 1);
        assert_eq!(
            hex::encode(&block.txs[0].tx.raw),
            json_block["tx"][0]["hex"]
        );
        assert_eq!(
            block.txs[0].tx.txid.to_hex_be(),
            json_block["tx"][0]["txid"]
        );
        let block_from_hash = rpc.get_block(BlockIdentifier::Hash(Sha256d::from_hex_be(
            &genesis_block_hash,
        )?))?;
        assert_eq!(block, block_from_hash);
        Ok(())
    }
}
