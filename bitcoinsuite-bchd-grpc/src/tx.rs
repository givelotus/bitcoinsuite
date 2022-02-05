use bitcoinsuite_core::{
    ByteArray, Bytes, Hashed, OutPoint, Script, SequenceNo, Sha256d, TxInput, TxOutput, UnhashedTx,
};
use bitcoinsuite_slp::{
    SlpAmount, SlpBurn, SlpGenesisInfo, SlpToken, SlpTx, SlpTxData, SlpTxType, TokenId,
};

use crate::bchd_grpc::{
    self,
    slp_transaction_info::{TxMetadata, ValidityJudgement},
    SlpAction,
};

pub fn to_slp_tx(tx: bchd_grpc::Transaction) -> SlpTx {
    let unhashed_tx = UnhashedTx {
        version: tx.version,
        inputs: tx
            .inputs
            .iter()
            .map(|input| {
                let outpoint = input.outpoint.as_ref().unwrap();
                TxInput {
                    prev_out: OutPoint {
                        txid: Sha256d::from_slice_or_null(&outpoint.hash),
                        out_idx: outpoint.index,
                    },
                    script: Script::from_slice(&input.signature_script),
                    sequence: SequenceNo::from_u32(input.sequence),
                    ..Default::default()
                }
            })
            .collect::<Vec<_>>(),
        outputs: tx
            .outputs
            .iter()
            .map(|output| TxOutput {
                value: output.value,
                script: Script::from_slice(&output.pubkey_script),
            })
            .collect::<Vec<_>>(),
        lock_time: tx.lock_time,
    };
    let mut token_id = None;
    let is_valid_slp = tx
        .slp_transaction_info
        .as_ref()
        .map(|slp| {
            token_id = Some(TokenId::from_slice_be_or_null(&slp.token_id));
            slp.validity_judgement() == ValidityJudgement::Valid
        })
        .unwrap_or_default();
    let mut input_tokens = Vec::with_capacity(tx.inputs.len());
    let mut burns = Vec::with_capacity(tx.inputs.len());
    for input in &tx.inputs {
        match (&input.slp_token, &token_id) {
            (Some(slp_token), Some(token_id))
                if is_valid_slp && slp_token.token_id == token_id.as_slice_be() =>
            {
                input_tokens.push(SlpToken {
                    amount: SlpAmount::new(slp_token.amount.into()),
                    is_mint_baton: slp_token.is_mint_baton,
                });
                burns.push(None);
            }
            (Some(slp_token), _) => {
                input_tokens.push(SlpToken::default());
                burns.push(Some(Box::new(SlpBurn {
                    token: SlpToken {
                        amount: SlpAmount::new(slp_token.amount.into()),
                        is_mint_baton: slp_token.is_mint_baton,
                    },
                    token_id: TokenId::from_slice_be_or_null(&slp_token.token_id),
                })));
            }
            _ => {
                input_tokens.push(SlpToken::default());
                burns.push(None);
            }
        }
    }
    let slp_tx_data = tx
        .slp_transaction_info
        .as_ref()
        .filter(|slp_tx_info| slp_tx_info.slp_action() != SlpAction::NonSlp)
        .map(|slp| SlpTxData {
            input_tokens,
            output_tokens: tx
                .outputs
                .iter()
                .map(|output| {
                    if let Some(slp_token) = &output.slp_token {
                        if is_valid_slp {
                            return SlpToken {
                                amount: SlpAmount::new(slp_token.amount.into()),
                                is_mint_baton: slp_token.is_mint_baton,
                            };
                        }
                    }
                    SlpToken::default()
                })
                .collect(),
            slp_tx_type: match &slp.tx_metadata {
                Some(TxMetadata::V1Genesis(genesis)) => {
                    SlpTxType::Genesis(Box::new(SlpGenesisInfo {
                        token_ticker: Bytes::from_slice(&genesis.ticker),
                        token_name: Bytes::from_slice(&genesis.name),
                        token_document_url: Bytes::from_slice(&genesis.document_url),
                        token_document_hash: genesis
                            .document_hash
                            .as_slice()
                            .try_into()
                            .ok()
                            .map(ByteArray::new),
                        decimals: genesis.decimals,
                    }))
                }
                Some(TxMetadata::V1Mint(_)) => SlpTxType::Mint,
                Some(TxMetadata::V1Send(_)) => SlpTxType::Send,
                _ => SlpTxType::Unknown,
            },
            token_id: TokenId::from_slice_be_or_null(&slp.token_id),
        });
    SlpTx::new(unhashed_tx, slp_tx_data, burns)
}
