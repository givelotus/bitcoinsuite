use crate::{
    lotus_txid, opcode::*, BitcoinBlock, BitcoinHeader, Bytes, BytesMut, LotusBlock, LotusHeader,
    LotusMetadataField, OutPoint, Script, SequenceNo, Sha256d, Tx, TxInput, TxOutput, UnhashedTx,
};

pub fn ser_script_num(value: i32) -> Bytes {
    let mut bytes = BytesMut::new();
    if value == 0 {
        return bytes.freeze();
    }
    let is_neg = value < 0;
    let mut abs_val = value.abs();
    while abs_val > 0 {
        bytes.put_slice(&[(abs_val & 0xff) as u8]);
        abs_val >>= 8;
    }
    if bytes.as_slice().last().unwrap() & 0x80 != 0 {
        bytes.put_slice(&[if is_neg { 0x80 } else { 0 }]);
    } else if is_neg {
        *bytes.as_slice_mut().last_mut().unwrap() |= 0x80;
    }
    bytes.freeze()
}

#[allow(clippy::inconsistent_digit_grouping)]
pub fn build_bitcoin_coinbase(height: i32, script: Script) -> UnhashedTx {
    let value = 50_000_000_00i64 >> (height / 150);
    let script_sig = match height {
        // These heights are never checked anyway, we can put junk in here
        0..=127 => Script::new([0xff, height as u8].into()),
        _ => {
            let mut script = BytesMut::new();
            let num = ser_script_num(height);
            script.put_slice(&[num.len() as u8]);
            script.put_bytes(num);
            Script::new(script.freeze())
        }
    };
    UnhashedTx {
        version: 1,
        inputs: vec![TxInput {
            prev_out: OutPoint {
                txid: Sha256d::new([0; 32]),
                out_idx: 0xffff_ffff,
            },
            script: script_sig,
            sequence: SequenceNo::finalized(),
            ..Default::default()
        }],
        outputs: vec![
            TxOutput { value, script },
            TxOutput {
                value: 0,
                script: Script::opreturn(&[&[0; 100]]),
            },
        ],
        lock_time: 0,
    }
}

pub fn build_lotus_coinbase(height: i32, script: Script) -> UnhashedTx {
    let value = 260_000_000;
    let opreturn = match height {
        0 => Script::from_static_slice(b"\x05logos\0"),
        1..=16 => Script::new(
            [[OP_RETURN].as_ref(), b"\x05logos", &[height as u8 + 0x50]]
                .concat()
                .into(),
        ),
        _ => Script::opreturn(&[b"logos".as_ref(), &ser_script_num(height)]),
    };
    UnhashedTx {
        version: 1,
        inputs: vec![TxInput {
            prev_out: OutPoint {
                txid: Sha256d::new([0; 32]),
                out_idx: 0xffff_ffff,
            },
            script: Script::new([0; 80].into()),
            sequence: SequenceNo::finalized(),
            ..Default::default()
        }],
        outputs: vec![
            TxOutput {
                value: 0,
                script: opreturn,
            },
            TxOutput { value, script },
        ],
        lock_time: 0,
    }
}

pub fn build_bitcoin_block(
    prev_block: Sha256d,
    timestamp: u32,
    coinbase: Tx,
    mut txs: Vec<Tx>,
) -> BitcoinBlock {
    txs.sort_unstable_by_key(|tx| tx.hash().clone());
    txs.insert(0, coinbase);
    let mut block = BitcoinBlock {
        header: BitcoinHeader {
            version: 2,
            bits: 0x207fffff,
            merkle_root: Sha256d::default(),
            prev_block,
            timestamp,
            nonce: 0,
        },
        txs,
    };
    block.update_merkle_root();
    block.header.solve();
    block
}

pub fn build_lotus_block(
    prev_block: Sha256d,
    timestamp: i64,
    height: i32,
    coinbase: Tx,
    mut txs: Vec<Tx>,
    epoch_hash: Sha256d,
    metadata: Vec<LotusMetadataField>,
) -> LotusBlock {
    txs.sort_unstable_by_key(|tx| lotus_txid(tx.unhashed_tx()));
    txs.insert(0, coinbase);
    let mut block = LotusBlock {
        header: LotusHeader {
            prev_block,
            bits: 0x207fffff,
            timestamp,
            version: 1,
            height,
            epoch_hash,
            ..Default::default()
        },
        metadata,
        txs,
    };
    block.prepare();
    block
}
