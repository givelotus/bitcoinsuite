use bitcoinsuite_error::{ErrorMeta, Result};
use flatbuffers::VerifierOptions;
use nng::{
    options::{
        protocol::pubsub::{Subscribe, Unsubscribe},
        Options,
    },
    Protocol, Socket,
};
use thiserror::Error;
use tokio::sync::mpsc;

use crate::{
    nng_interface_generated::nng_interface::{
        BlockConnected, BlockDisconnected, ChainStateFlushed, TransactionAddedToMempool,
        TransactionRemovedFromMempool, UpdatedBlockTip,
    },
    structs,
};

#[derive(Clone)]
pub struct PubInterface {
    sock: Socket,
    fbb_opts: VerifierOptions,
}

#[derive(Error, Debug, ErrorMeta)]
pub enum PubInterfaceError {
    #[bug()]
    #[error("Invalid pub message: {0:?}")]
    InvalidPubMessage(nng::Message),

    #[bug()]
    #[error("No message received by NNG")]
    NoMessageReceived,
}

use self::PubInterfaceError::*;

impl PubInterface {
    pub fn open(pub_url: &str) -> Result<Self> {
        let sock = Socket::new(Protocol::Sub0)?;
        sock.dial(pub_url)?;
        Ok(PubInterface {
            sock,
            fbb_opts: VerifierOptions {
                max_tables: 0xffff_ffff,
                ..Default::default()
            },
        })
    }

    pub fn subscribe(&self, topic: &str) -> Result<()> {
        self.sock.set_opt::<Subscribe>(topic.as_bytes().to_vec())?;
        Ok(())
    }

    pub fn unsubscribe(&self, topic: &str) -> Result<()> {
        self.sock
            .set_opt::<Unsubscribe>(topic.as_bytes().to_vec())?;
        Ok(())
    }

    pub async fn recv_async(&self) -> Result<structs::Message> {
        let (sender, mut receiver) = mpsc::unbounded_channel::<nng::AioResult>();
        let aio = nng::Aio::new(move |_, result| {
            sender.send(result).unwrap();
        })?;
        self.sock.recv_async(&aio)?;
        let result = receiver.recv().await.ok_or(NoMessageReceived)?;
        let msg = match result {
            nng::AioResult::Recv(msg) => msg?,
            _ => unreachable!(),
        };
        self.parse_msg(msg)
    }

    pub fn recv(&self) -> Result<structs::Message> {
        let msg = self.sock.recv()?;
        self.parse_msg(msg)
    }

    fn parse_msg(&self, msg: nng::Message) -> Result<structs::Message> {
        const PREFIX_LEN: usize = 12;
        if msg.len() < PREFIX_LEN {
            eprintln!("Message has invalid length: {}", msg.len());
            return Err(InvalidPubMessage(msg).into());
        }
        let prefix = &msg[..PREFIX_LEN];
        let payload = &msg[PREFIX_LEN..];
        Ok(match prefix {
            b"updateblktip" => {
                let msg = flatbuffers::root_with_opts::<UpdatedBlockTip>(&self.fbb_opts, payload)?;
                structs::Message::UpdatedBlockTip(structs::UpdatedBlockTip::from_fbs(msg)?)
            }
            b"mempooltxadd" => {
                let msg = flatbuffers::root_with_opts::<TransactionAddedToMempool>(
                    &self.fbb_opts,
                    payload,
                )?;
                structs::Message::TransactionAddedToMempool(
                    structs::TransactionAddedToMempool::from_fbs(msg)?,
                )
            }
            b"mempooltxrem" => {
                let msg = flatbuffers::root_with_opts::<TransactionRemovedFromMempool>(
                    &self.fbb_opts,
                    payload,
                )?;
                structs::Message::TransactionRemovedFromMempool(
                    structs::TransactionRemovedFromMempool::from_fbs(msg)?,
                )
            }
            b"blkconnected" => {
                let msg = flatbuffers::root_with_opts::<BlockConnected>(&self.fbb_opts, payload)?;
                structs::Message::BlockConnected(structs::BlockConnected::from_fbs(msg)?)
            }
            b"blkdisconctd" => {
                let msg =
                    flatbuffers::root_with_opts::<BlockDisconnected>(&self.fbb_opts, payload)?;
                structs::Message::BlockDisconnected(structs::BlockDisconnected::from_fbs(msg)?)
            }
            b"chainstflush" => {
                let msg =
                    flatbuffers::root_with_opts::<ChainStateFlushed>(&self.fbb_opts, payload)?;
                structs::Message::ChainStateFlushed(structs::ChainStateFlushed::from_fbs(msg)?)
            }
            _ => {
                eprintln!("Unknown message prefix: {prefix:?}");
                return Err(InvalidPubMessage(msg).into());
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, str::FromStr};

    use bitcoinsuite_bitcoind::instance::{BitcoindChain, BitcoindConf, BitcoindInstance};
    use bitcoinsuite_core::{AddressType, CashAddress, Hashed, ShaRmd160, BCHREG};
    use bitcoinsuite_error::Result;
    use bitcoinsuite_test_utils::bin_folder;
    use tempdir::TempDir;

    use crate::{Message, PubInterface};

    #[tokio::test]
    async fn test_pub() -> Result<()> {
        bitcoinsuite_error::install()?;
        let ipc_dir = TempDir::new("ipc_pub_dir")?;
        let pub_url = format!(
            "ipc://{}",
            ipc_dir.path().join("pub.pipe").to_string_lossy()
        );
        let conf = BitcoindConf::from_chain_regtest(
            bin_folder(),
            BitcoindChain::XPI,
            vec![
                OsString::from_str(&format!("-nngpub={pub_url}"))?,
                OsString::from_str("-nngpubmsg=updateblktip")?,
            ],
        )?;
        let mut instance = BitcoindInstance::setup(conf)?;
        instance.wait_for_ready()?;
        let pub_interface = PubInterface::open(&pub_url)?;
        test_update_block_tip(&mut instance, &pub_interface).await?;
        instance.cleanup()?;
        Ok(())
    }

    async fn test_update_block_tip(
        instance: &mut BitcoindInstance,
        pub_interface: &PubInterface,
    ) -> Result<()> {
        pub_interface.subscribe("updateblktip")?;
        let address = CashAddress::from_hash(BCHREG, AddressType::P2SH, ShaRmd160::new([0; 20]));
        {
            let hashes = instance.cmd_json("generatetoaddress", &["1", address.as_str()])?;
            let msg = pub_interface.recv()?;
            match msg {
                Message::UpdatedBlockTip(msg) => {
                    assert_eq!(&msg.block_hash.to_hex_be(), hashes[0].as_str().unwrap());
                }
                _ => panic!("Invalid message received"),
            }
        }
        {
            let hashes = instance.cmd_json("generatetoaddress", &["1", address.as_str()])?;
            let msg = pub_interface.recv_async().await?;
            match msg {
                Message::UpdatedBlockTip(msg) => {
                    assert_eq!(&msg.block_hash.to_hex_be(), hashes[0].as_str().unwrap());
                }
                _ => panic!("Invalid message received"),
            }
        }
        Ok(())
    }
}
