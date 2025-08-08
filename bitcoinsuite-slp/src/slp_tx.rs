use bitcoinsuite_core::{ByteArray, Bytes, Script, TxInput, TxOutput, UnhashedTx};
use serde::{Deserialize, Serialize};

use crate::{
    consts::{SLP_TOKEN_TYPE_V1, SLP_TOKEN_TYPE_V1_NFT1_CHILD, SLP_TOKEN_TYPE_V1_NFT1_GROUP},
    SlpAmount, TokenId,
};

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
    pub slp_token_type: SlpTokenType,
    pub slp_tx_type: SlpTxType,
    /// 0000...000000 if token_id is incomplete
    pub token_id: TokenId,
    pub group_token_id: Option<Box<TokenId>>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlpTokenType {
    Fungible,
    Nft1Group,
    Nft1Child,
    Unknown,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SlpTxType {
    Genesis(Box<SlpGenesisInfo>),
    Send,
    Mint,
    Burn(u64),
    Unknown,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SlpTxTypeVariant {
    Genesis,
    Send,
    Mint,
    Burn,
    Unknown,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash, Default)]
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
            SlpTxType::Send => "SEND",
            SlpTxType::Burn(_) => "BURN",
            SlpTxType::Unknown => "UNKNOWN",
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
        }
        SlpTx {
            tx,
            slp_tx_data: slp_tx_data.map(Box::new),
            slp_burns,
        }
    }

    pub fn inputs(&self) -> impl Iterator<Item = SlpTxInput<'_>> {
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

    pub fn outputs(&self) -> impl Iterator<Item = SlpTxOutput<'_>> {
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

impl SlpToken {
    pub const MINT_BATON: SlpToken = SlpToken {
        amount: SlpAmount::ZERO,
        is_mint_baton: true,
    };

    pub const EMPTY: SlpToken = SlpToken {
        amount: SlpAmount::ZERO,
        is_mint_baton: false,
    };

    pub const fn amount(amount: i128) -> Self {
        SlpToken {
            amount: SlpAmount::new(amount),
            is_mint_baton: false,
        }
    }
}

impl SlpTxType {
    pub fn tx_type_variant(&self) -> SlpTxTypeVariant {
        match &self {
            SlpTxType::Genesis(_) => SlpTxTypeVariant::Genesis,
            SlpTxType::Send => SlpTxTypeVariant::Send,
            SlpTxType::Mint => SlpTxTypeVariant::Mint,
            SlpTxType::Burn(_) => SlpTxTypeVariant::Burn,
            SlpTxType::Unknown => SlpTxTypeVariant::Unknown,
        }
    }
}

impl SlpTokenType {
    pub fn to_u16(&self) -> Option<u16> {
        match self {
            SlpTokenType::Fungible => Some(SLP_TOKEN_TYPE_V1[0] as _),
            SlpTokenType::Nft1Group => Some(SLP_TOKEN_TYPE_V1_NFT1_GROUP[0] as _),
            SlpTokenType::Nft1Child => Some(SLP_TOKEN_TYPE_V1_NFT1_CHILD[0] as _),
            SlpTokenType::Unknown => None,
        }
    }
}
