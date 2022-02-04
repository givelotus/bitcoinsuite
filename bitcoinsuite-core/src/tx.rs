use crate::{BitcoinCode, Bytes, BytesMut, Hashed, Result, Script, SequenceNo, Sha256d, SignData};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnhashedTx {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tx {
    unhashed_tx: UnhashedTx,
    hash: Sha256d,
    raw: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct OutPoint {
    pub txid: Sha256d,
    pub out_idx: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TxInput {
    pub prev_out: OutPoint,
    pub script: Script,
    pub sequence: SequenceNo,
    pub sign_data: Option<SignData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TxOutput {
    pub value: i64,
    pub script: Script,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Coin {
    pub tx_output: TxOutput,
    pub height: Option<i32>,
    pub is_coinbase: bool,
}

impl UnhashedTx {
    pub fn hashed(self) -> Tx {
        let mut data = BytesMut::new();
        self.ser_to(&mut data);
        let raw = data.freeze();
        let hash = Sha256d::digest(raw.clone());
        Tx {
            unhashed_tx: self,
            hash,
            raw,
        }
    }
}

impl Tx {
    pub fn hash(&self) -> &Sha256d {
        &self.hash
    }

    pub fn raw(&self) -> &Bytes {
        &self.raw
    }

    pub fn version(&self) -> i32 {
        self.unhashed_tx.version
    }

    pub fn inputs(&self) -> &[TxInput] {
        &self.unhashed_tx.inputs
    }

    pub fn outputs(&self) -> &[TxOutput] {
        &self.unhashed_tx.outputs
    }

    pub fn lock_time(&self) -> u32 {
        self.unhashed_tx.lock_time
    }

    pub fn unhashed_tx(&self) -> &UnhashedTx {
        &self.unhashed_tx
    }

    pub fn into_unhashed_tx(self) -> UnhashedTx {
        self.unhashed_tx
    }
}

impl OutPoint {
    pub fn is_coinbase(&self) -> bool {
        self.txid == Sha256d::new([0; 32]) && self.out_idx == 0xffff_ffff
    }
}

impl BitcoinCode for UnhashedTx {
    fn ser_to(&self, bytes: &mut BytesMut) {
        self.version.ser_to(bytes);
        self.inputs.ser_to(bytes);
        self.outputs.ser_to(bytes);
        self.lock_time.ser_to(bytes);
    }

    fn deser(data: &mut Bytes) -> Result<Self> {
        Ok(UnhashedTx {
            version: BitcoinCode::deser(data)?,
            inputs: BitcoinCode::deser(data)?,
            outputs: BitcoinCode::deser(data)?,
            lock_time: BitcoinCode::deser(data)?,
        })
    }
}

impl BitcoinCode for Tx {
    fn ser_to(&self, bytes: &mut BytesMut) {
        self.unhashed_tx.ser_to(bytes)
    }

    fn deser(data: &mut Bytes) -> Result<Self> {
        Ok(UnhashedTx::deser(data)?.hashed())
    }
}

impl BitcoinCode for OutPoint {
    fn ser_to(&self, bytes: &mut BytesMut) {
        self.txid.ser_to(bytes);
        self.out_idx.ser_to(bytes);
    }

    fn deser(data: &mut Bytes) -> Result<Self> {
        Ok(OutPoint {
            txid: BitcoinCode::deser(data)?,
            out_idx: BitcoinCode::deser(data)?,
        })
    }
}

impl BitcoinCode for TxInput {
    fn ser_to(&self, bytes: &mut BytesMut) {
        self.prev_out.ser_to(bytes);
        self.script.ser_to(bytes);
        self.sequence.ser_to(bytes);
    }

    fn deser(data: &mut Bytes) -> Result<Self> {
        Ok(TxInput {
            prev_out: BitcoinCode::deser(data)?,
            script: BitcoinCode::deser(data)?,
            sequence: BitcoinCode::deser(data)?,
            ..Default::default()
        })
    }
}

impl BitcoinCode for TxOutput {
    fn ser_to(&self, bytes: &mut BytesMut) {
        self.value.ser_to(bytes);
        self.script.ser_to(bytes);
    }

    fn deser(data: &mut Bytes) -> Result<Self> {
        Ok(TxOutput {
            value: BitcoinCode::deser(data)?,
            script: BitcoinCode::deser(data)?,
        })
    }
}
