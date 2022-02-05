use bitcoinsuite_core::{ByteArray, Bytes, Script, TxInput, TxOutput, UnhashedTx};

use crate::{SlpAmount, TokenId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlpTx {
    tx: UnhashedTx,
    slp_tx_data: Option<Box<SlpTxData>>,
    slp_burns: Vec<Option<Box<SlpBurn>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlpTxData {
    pub input_tokens: Vec<SlpToken>,
    pub output_tokens: Vec<SlpToken>,
    pub slp_tx_type: SlpTxType,
    /// 0000...000000 for unknown token types
    pub token_id: TokenId,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SlpTxType {
    Genesis(Box<SlpGenesisInfo>),
    Send,
    Mint,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SlpGenesisInfo {
    pub token_ticker: Bytes,
    pub token_name: Bytes,
    pub token_document_url: Bytes,
    pub token_document_hash: Option<ByteArray<32>>,
    pub decimals: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlpTxInput<'tx> {
    pub token: &'tx SlpToken,
    pub input: &'tx TxInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlpBurn {
    pub token: SlpToken,
    pub token_id: TokenId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlpTxOutput<'tx> {
    pub token: &'tx SlpToken,
    pub output: &'tx TxOutput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlpSendOutput {
    pub amount: SlpAmount,
    pub script: Script,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SlpToken {
    pub amount: SlpAmount,
    pub is_mint_baton: bool,
}

impl SlpTxType {
    pub fn type_str(&self) -> &'static str {
        match self {
            SlpTxType::Genesis(_) => "GENESIS",
            SlpTxType::Mint => "MINT",
            Self::Send => "SEND",
            Self::Unknown => "UNKNOWN",
        }
    }
}

impl SlpTx {
    pub fn new(
        tx: UnhashedTx,
        slp_tx_data: Option<SlpTxData>,
        slp_burns: Vec<Option<Box<SlpBurn>>>,
    ) -> Self {
        if let Some(slp_tx_data) = &slp_tx_data {
            if slp_tx_data.input_tokens.len() != tx.inputs.len() {
                panic!(
                    "tx inputs and slp data input tokens have inconsistent length: {} != {}",
                    tx.inputs.len(),
                    slp_tx_data.input_tokens.len()
                );
            }
            if slp_tx_data.output_tokens.len() != tx.outputs.len() {
                panic!(
                    "tx outputs and slp data output tokens have inconsistent length: {} != {}",
                    tx.outputs.len(),
                    slp_tx_data.output_tokens.len()
                );
            }
        }
        SlpTx {
            tx,
            slp_tx_data: slp_tx_data.map(Box::new),
            slp_burns,
        }
    }

    pub fn inputs(&self) -> impl Iterator<Item = SlpTxInput> {
        let input_tokens = match &self.slp_tx_data {
            Some(slp_tx_data) => slp_tx_data.input_tokens.as_slice(),
            None => &[],
        };
        self.tx
            .inputs
            .iter()
            .zip(input_tokens)
            .map(|(input, token)| SlpTxInput { input, token })
    }

    pub fn outputs(&self) -> impl Iterator<Item = SlpTxOutput> {
        let output_tokens = match &self.slp_tx_data {
            Some(slp_tx_data) => slp_tx_data.output_tokens.as_slice(),
            None => &[],
        };
        self.tx
            .outputs
            .iter()
            .zip(output_tokens)
            .map(|(output, token)| SlpTxOutput { output, token })
    }

    pub fn tx(&self) -> &UnhashedTx {
        &self.tx
    }

    pub fn slp(&self) -> Option<&SlpTxData> {
        self.slp_tx_data.as_deref()
    }

    pub fn burns(&self) -> &[Option<Box<SlpBurn>>] {
        &self.slp_burns
    }
}
