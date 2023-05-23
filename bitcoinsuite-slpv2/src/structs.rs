use std::ops::Range;

use bitcoinsuite_core::Bytes;

use crate::TokenId;

pub type LokadId = [u8; 4];
pub type Amount = i64;

pub const SLPV2_LOKAD_ID: LokadId = *b"SLP2";
pub const DEFAULT_TOKEN_TYPE: u8 = 200;

pub const GENESIS: &[u8] = b"GENESIS";
pub const MINT: &[u8] = b"MINT";
pub const SEND: &[u8] = b"SEND";
pub const BURN: &[u8] = b"BURN";

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum TokenType {
    Standard = DEFAULT_TOKEN_TYPE,
}

#[derive(Clone, Debug)]
pub struct TokenMeta {
    pub token_id: TokenId,
    pub token_type: TokenType,
}

#[derive(Debug)]
pub struct Section {
    pub meta: TokenMeta,
    pub variant: SectionVariant,
}

#[derive(Debug)]
pub enum SectionVariant {
    Genesis(Genesis),
    Mint(MintData),
    Send(Send),
}

#[derive(Clone, Debug, Default)]
pub struct GenesisData {
    pub token_ticker: Bytes,
    pub token_name: Bytes,
    pub url: Bytes,
    pub data: Bytes,
    pub auth_pubkey: Bytes,
    pub decimals: u8,
}

#[derive(Clone, Debug)]
pub struct Genesis {
    pub data: GenesisData,
    pub mint_data: MintData,
}

#[derive(Clone, Debug)]
pub struct MintData {
    pub amounts: Vec<Amount>,
    pub num_batons: usize,
}

#[derive(Clone, Debug)]
pub struct Send {
    pub output_amounts: Vec<Amount>,
    pub intentional_burn_amount: Option<Amount>,
}

#[derive(Debug, Default)]
pub struct Parsed {
    pub sections: Vec<Section>,
}

pub struct TokenAmount<'a> {
    pub token_id: &'a TokenId,
    pub amount: Amount,
}

impl MintData {
    pub fn amounts_range(&self) -> Range<usize> {
        1..1 + self.amounts.len()
    }

    pub fn batons_range(&self) -> Range<usize> {
        let start = 1 + self.amounts.len();
        start..start + self.num_batons
    }
}
