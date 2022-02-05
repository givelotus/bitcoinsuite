use crate::{
    get_merkle_root, BitcoinCode, Bytes, BytesMut, Hashed, MerkleMode, Result, Sha256d, Tx,
};

pub struct BitcoinHeader {
    pub version: i32,
    pub prev_block: Sha256d,
    pub merkle_root: Sha256d,
    pub timestamp: u32,
    pub bits: u32,
    pub nonce: u32,
}

pub struct BitcoinBlock {
    pub header: BitcoinHeader,
    pub txs: Vec<Tx>,
}

impl BitcoinCode for BitcoinHeader {
    fn ser_to(&self, bytes: &mut BytesMut) {
        self.version.ser_to(bytes);
        self.prev_block.ser_to(bytes);
        self.merkle_root.ser_to(bytes);
        self.timestamp.ser_to(bytes);
        self.bits.ser_to(bytes);
        self.nonce.ser_to(bytes);
    }

    fn deser(data: &mut Bytes) -> Result<Self> {
        Ok(BitcoinHeader {
            version: BitcoinCode::deser(data)?,
            prev_block: BitcoinCode::deser(data)?,
            merkle_root: BitcoinCode::deser(data)?,
            timestamp: BitcoinCode::deser(data)?,
            bits: BitcoinCode::deser(data)?,
            nonce: BitcoinCode::deser(data)?,
        })
    }
}

impl BitcoinCode for BitcoinBlock {
    fn ser_to(&self, bytes: &mut BytesMut) {
        self.header.ser_to(bytes);
        self.txs.ser_to(bytes);
    }

    fn deser(data: &mut Bytes) -> Result<Self> {
        Ok(BitcoinBlock {
            header: BitcoinCode::deser(data)?,
            txs: BitcoinCode::deser(data)?,
        })
    }
}

impl BitcoinBlock {
    pub fn update_merkle_root(&mut self) {
        let leaves = self.txs.iter().map(|tx| tx.hash().clone()).collect();
        self.header.merkle_root = get_merkle_root(leaves, MerkleMode::Bitcoin);
    }
}

impl BitcoinHeader {
    pub fn calc_hash(&self) -> Sha256d {
        let mut bytes = BytesMut::new();
        self.ser_to(&mut bytes);
        Sha256d::digest(bytes.freeze())
    }
}
