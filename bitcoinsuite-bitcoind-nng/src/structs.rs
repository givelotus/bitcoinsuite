use bitcoinsuite_core::{Coin, Sha256d};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tx {
    pub txid: Sha256d,
    pub raw: Vec<u8>,
    pub spent_coins: Option<Vec<Coin>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockHeader {
    pub raw: Vec<u8>,
    pub hash: Sha256d,
    pub prev_hash: Sha256d,
    pub n_bits: u32,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub header: BlockHeader,
    pub metadata: Vec<BlockMetadata>,
    pub txs: Vec<BlockTx>,
    pub file_num: u32,
    pub data_pos: u32,
    pub undo_pos: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockTx {
    pub tx: Tx,
    pub data_pos: u32,
    pub undo_pos: u32,
    pub undo_size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MempoolTx {
    pub tx: Tx,
    pub time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockMetadata {
    pub field_id: u32,
    pub field_value: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockIdentifier {
    Height(i32),
    Hash(Sha256d),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    UpdatedBlockTip(UpdatedBlockTip),
    TransactionAddedToMempool(TransactionAddedToMempool),
    TransactionRemovedFromMempool(TransactionRemovedFromMempool),
    BlockConnected(BlockConnected),
    BlockDisconnected(BlockDisconnected),
    ChainStateFlushed(ChainStateFlushed),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdatedBlockTip {
    pub block_hash: Sha256d,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionAddedToMempool {
    pub mempool_tx: MempoolTx,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionRemovedFromMempool {
    pub txid: Sha256d,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockConnected {
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockDisconnected {
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainStateFlushed {
    pub block_hash: Sha256d,
}
