use std::convert::TryInto;

use bitcoinsuite_core::Script;

use crate::{
    consts::{SLP_LOKAD_ID, SLP_TOKEN_TYPE_V1},
    SlpAmount, SlpGenesisInfo, TokenId,
};

pub fn genesis_opreturn(
    genesis_info: &SlpGenesisInfo,
    mint_baton_out_idx: Option<usize>,
    initial_quantity: u64,
) -> Script {
    Script::opreturn(&[
        SLP_LOKAD_ID,
        SLP_TOKEN_TYPE_V1,
        b"GENESIS",
        &genesis_info.token_ticker,
        &genesis_info.token_name,
        &genesis_info.token_document_url,
        match &genesis_info.token_document_hash {
            Some(hash) => hash,
            None => &[],
        },
        &[genesis_info.decimals.try_into().unwrap()],
        &match mint_baton_out_idx {
            Some(out_idx) => vec![out_idx.try_into().unwrap()],
            None => vec![],
        },
        &initial_quantity.to_be_bytes(),
    ])
}

pub fn mint_opreturn(
    token_id: &TokenId,
    mint_baton_out_idx: Option<usize>,
    additional_quantity: u64,
) -> Script {
    Script::opreturn(&[
        SLP_LOKAD_ID,
        SLP_TOKEN_TYPE_V1,
        b"MINT",
        token_id.as_slice_be(),
        &match mint_baton_out_idx {
            Some(out_idx) => vec![out_idx.try_into().unwrap()],
            None => vec![],
        },
        &additional_quantity.to_be_bytes(),
    ])
}

pub fn send_opreturn(token_id: &TokenId, send_amounts: &[SlpAmount]) -> Script {
    let mut pushes: Vec<&[u8]> = vec![
        SLP_LOKAD_ID,
        SLP_TOKEN_TYPE_V1,
        b"SEND",
        token_id.as_slice_be(),
    ];
    let send_amounts = send_amounts
        .iter()
        .map(|amount| amount.base_amount() as u64)
        .map(|amount| amount.to_be_bytes())
        .collect::<Vec<_>>();
    pushes.extend(send_amounts.iter().map(|slice| slice.as_ref()));
    Script::opreturn(&pushes)
}
