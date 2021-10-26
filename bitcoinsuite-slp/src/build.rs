use bitcoinsuite_core::Script;

use crate::{
    consts::{
        SLP_LOKAD_ID, SLP_TOKEN_TYPE_V1, SLP_TOKEN_TYPE_V1_NFT1_CHILD, SLP_TOKEN_TYPE_V1_NFT1_GROUP,
    },
    SlpAmount, SlpGenesisInfo, SlpTokenType, TokenId,
};

fn token_type_bytes(token_type: SlpTokenType) -> &'static [u8] {
    match token_type {
        SlpTokenType::Fungible => SLP_TOKEN_TYPE_V1,
        SlpTokenType::Nft1Group => SLP_TOKEN_TYPE_V1_NFT1_GROUP,
        SlpTokenType::Nft1Child => SLP_TOKEN_TYPE_V1_NFT1_CHILD,
        SlpTokenType::Unknown => panic!("Cannot use 'Unknown' token type here"),
    }
}

pub fn genesis_opreturn(
    genesis_info: &SlpGenesisInfo,
    token_type: SlpTokenType,
    mint_baton_out_idx: Option<usize>,
    initial_quantity: u64,
) -> Script {
    Script::opreturn(&[
        SLP_LOKAD_ID,
        token_type_bytes(token_type),
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
    token_type: SlpTokenType,
    mint_baton_out_idx: Option<usize>,
    additional_quantity: u64,
) -> Script {
    Script::opreturn(&[
        SLP_LOKAD_ID,
        token_type_bytes(token_type),
        b"MINT",
        token_id.as_slice_be(),
        &match mint_baton_out_idx {
            Some(out_idx) => vec![out_idx.try_into().unwrap()],
            None => vec![],
        },
        &additional_quantity.to_be_bytes(),
    ])
}

pub fn send_opreturn(
    token_id: &TokenId,
    token_type: SlpTokenType,
    send_amounts: &[SlpAmount],
) -> Script {
    let mut pushes: Vec<&[u8]> = vec![
        SLP_LOKAD_ID,
        token_type_bytes(token_type),
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
