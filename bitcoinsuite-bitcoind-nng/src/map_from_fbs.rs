use bitcoinsuite_core::{Script, Sha256d, TxOutput};
use bitcoinsuite_error::Result;

use crate::{
    field::OptionExt,
    nng_interface_generated::nng_interface::{
        Block, BlockConnected, BlockDisconnected, BlockHash, BlockHeader, BlockMetadata, BlockTx,
        ChainStateFlushed, Coin, MempoolTx, TransactionAddedToMempool,
        TransactionRemovedFromMempool, Tx, TxId, UpdatedBlockTip,
    },
    structs,
};

fn hash_from_block_hash(fbs: BlockHash) -> Result<Sha256d> {
    Ok(Sha256d::new(fbs.hash().field("BlockHash.hash")?.0))
}

fn hash_from_txid(fbs: TxId) -> Result<Sha256d> {
    Ok(Sha256d::new(fbs.hash().field("TxId.hash")?.0))
}

fn coin_from_fbs(coin: Coin) -> Result<bitcoinsuite_core::Coin> {
    let tx_out = coin.tx_out().field("Coin.tx_out")?;
    Ok(bitcoinsuite_core::Coin {
        tx_output: TxOutput {
            value: tx_out.amount() as i64,
            script: Script::from_slice(tx_out.script().field("TxOut.script")?),
        },
        height: if coin.height() == -1 {
            None
        } else {
            Some(coin.height())
        },
        is_coinbase: coin.is_coinbase(),
    })
}

impl structs::Tx {
    pub fn from_fbs(fbs: Tx) -> Result<Self> {
        Ok(structs::Tx {
            txid: Sha256d::new(fbs.txid().field("Tx.txid")?.hash().field("Tx.txid.hash")?.0),
            raw: fbs.raw().field("Tx.raw")?.to_vec(),
            spent_coins: fbs
                .spent_coins()
                .map(|outputs| outputs.iter().map(coin_from_fbs).collect::<Result<_>>())
                .transpose()?,
        })
    }
}

impl structs::BlockHeader {
    pub fn from_fbs(fbs: BlockHeader) -> Result<Self> {
        Ok(structs::BlockHeader {
            raw: fbs.raw().field("BlockHeader.raw")?.to_vec(),
            hash: hash_from_block_hash(fbs.block_hash().field("BlockHeader.block_hash")?)?,
            prev_hash: hash_from_block_hash(
                fbs.prev_block_hash().field("BlockHeader.prev_block_hash")?,
            )?,
            n_bits: fbs.n_bits(),
            timestamp: fbs.timestamp(),
        })
    }
}

impl structs::Block {
    pub fn from_fbs(fbs: Block) -> Result<Self> {
        Ok(structs::Block {
            header: structs::BlockHeader::from_fbs(fbs.header().field("Block.header")?)?,
            metadata: fbs
                .metadata()
                .field("Block.metadata")?
                .into_iter()
                .map(structs::BlockMetadata::from_fbs)
                .collect::<Result<_>>()?,
            txs: fbs
                .txs()
                .field("Block.txs")?
                .into_iter()
                .map(structs::BlockTx::from_fbs)
                .collect::<Result<_>>()?,
            file_num: fbs.file_num(),
            data_pos: fbs.data_pos(),
            undo_pos: fbs.undo_pos(),
        })
    }
}

impl structs::MempoolTx {
    pub fn from_fbs(fbs: MempoolTx) -> Result<Self> {
        Ok(structs::MempoolTx {
            tx: structs::Tx::from_fbs(fbs.tx().field("MempoolTx.tx")?)?,
            time: fbs.time(),
        })
    }
}

impl structs::BlockTx {
    pub fn from_fbs(fbs: BlockTx) -> Result<Self> {
        Ok(structs::BlockTx {
            tx: structs::Tx::from_fbs(fbs.tx().field("BlockTx.tx")?)?,
            data_pos: fbs.data_pos(),
            undo_pos: fbs.undo_pos(),
            undo_size: fbs.undo_size(),
        })
    }
}

impl structs::BlockMetadata {
    pub fn from_fbs(fbs: BlockMetadata) -> Result<Self> {
        Ok(structs::BlockMetadata {
            field_id: fbs.field_id(),
            field_value: fbs
                .field_value()
                .field("BlockMetadata.field_value")?
                .to_vec(),
        })
    }
}

impl structs::UpdatedBlockTip {
    pub fn from_fbs(fbs: UpdatedBlockTip) -> Result<Self> {
        Ok(structs::UpdatedBlockTip {
            block_hash: hash_from_block_hash(
                fbs.block_hash().field("UpdatedBlockTip.block_hash")?,
            )?,
        })
    }
}

impl structs::TransactionAddedToMempool {
    pub fn from_fbs(fbs: TransactionAddedToMempool) -> Result<Self> {
        Ok(structs::TransactionAddedToMempool {
            mempool_tx: structs::MempoolTx::from_fbs(
                fbs.mempool_tx()
                    .field("TransactionAddedToMempool.mempool_tx")?,
            )?,
        })
    }
}

impl structs::TransactionRemovedFromMempool {
    pub fn from_fbs(fbs: TransactionRemovedFromMempool) -> Result<Self> {
        Ok(structs::TransactionRemovedFromMempool {
            txid: hash_from_txid(fbs.txid().field("TransactionRemovedFromMempool.txid")?)?,
        })
    }
}

impl structs::BlockConnected {
    pub fn from_fbs(fbs: BlockConnected) -> Result<Self> {
        Ok(structs::BlockConnected {
            block: structs::Block::from_fbs(fbs.block().field("BlockConnected.block")?)?,
        })
    }
}

impl structs::BlockDisconnected {
    pub fn from_fbs(fbs: BlockDisconnected) -> Result<Self> {
        Ok(structs::BlockDisconnected {
            block: structs::Block::from_fbs(fbs.block().field("BlockDisconnected.block")?)?,
        })
    }
}

impl structs::ChainStateFlushed {
    pub fn from_fbs(fbs: ChainStateFlushed) -> Result<Self> {
        Ok(structs::ChainStateFlushed {
            block_hash: hash_from_block_hash(
                fbs.block_hash().field("ChainStateFlushed.block_hash")?,
            )?,
        })
    }
}
