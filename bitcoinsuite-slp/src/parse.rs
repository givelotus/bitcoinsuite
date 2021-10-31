use bitcoinsuite_core::{ByteArray, Bytes, Op, Sha256d, UnhashedTx};

use crate::{
    consts::{
        SLP_LOKAD_ID, SLP_OUTPUT_QUANTITY_FIELD_NAMES, SLP_TOKEN_TYPE_V1,
        SLP_TOKEN_TYPE_V1_NFT1_CHILD, SLP_TOKEN_TYPE_V1_NFT1_GROUP,
    },
    SlpAmount, SlpError, SlpGenesisInfo, SlpToken, SlpTokenType, SlpTxType, TokenId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlpParseData {
    pub output_tokens: Vec<SlpToken>,
    pub slp_token_type: SlpTokenType,
    pub slp_tx_type: SlpTxType,
    /// 0000...000000 if token_id is incomplete
    pub token_id: TokenId,
}

pub fn parse_slp_tx(txid: &Sha256d, tx: &UnhashedTx) -> Result<SlpParseData, SlpError> {
    if tx.outputs.is_empty() {
        return Err(SlpError::NoOutputs);
    }
    let ops = tx.outputs[0].script.ops().collect::<Result<Vec<_>, _>>()?;
    let opreturn_data = parse_opreturn_ops(ops.into_iter())?;
    if opreturn_data.len() < 3 {
        return Err(SlpError::TooFewPushes {
            actual: opreturn_data.len(),
            expected: 3,
        });
    }
    parse_lokad_id(&opreturn_data[0])?;
    if opreturn_data[1].is_empty() || opreturn_data[1].len() > 2 {
        return Err(SlpError::InvalidTokenType(opreturn_data[1].clone()));
    }
    // Short circuit for unknown/unsupported token types
    let slp_token_type = match parse_token_type(&opreturn_data[1]) {
        Some(token_type) => token_type,
        None => {
            let token = SlpToken::EMPTY;
            return Ok(SlpParseData {
                output_tokens: tx.outputs.iter().map(|_| token).collect(),
                slp_token_type: SlpTokenType::Unknown,
                slp_tx_type: SlpTxType::Unknown,
                token_id: TokenId::new(Sha256d::new([0; 32])),
            });
        }
    };

    let parsed_opreturn = match opreturn_data[2].as_ref() {
        b"GENESIS" => parse_genesis_data(opreturn_data, slp_token_type)?,
        b"MINT" => parse_mint_data(opreturn_data)?,
        b"SEND" => parse_send_data(opreturn_data)?,
        _ => return Err(SlpError::InvalidTxType(opreturn_data[2].clone())),
    };
    let token_id = match (&parsed_opreturn.slp_tx_type, parsed_opreturn.token_id) {
        (SlpTxType::Genesis(_), None) => TokenId::new(txid.clone()),
        (SlpTxType::Mint | SlpTxType::Send, Some(expected_token_id)) => expected_token_id,
        _ => unreachable!(),
    };
    let mut output_tokens = tx
        .outputs
        .iter()
        .map(|_| SlpToken::EMPTY)
        .collect::<Vec<_>>();
    match parsed_opreturn.outputs {
        ParsedOutputs::MintTokens {
            mint_quantity,
            baton_out_idx,
        } => {
            if let Some(baton_out_idx) = baton_out_idx {
                if let Some(output_token) = output_tokens.get_mut(baton_out_idx) {
                    output_token.is_mint_baton = true;
                }
            }
            if let Some(output_token) = output_tokens.get_mut(1) {
                output_token.amount = mint_quantity;
            }
        }
        ParsedOutputs::Send(amounts) => {
            output_tokens.resize(amounts.len() + 1, SlpToken::EMPTY);
            for (output_token, amount) in output_tokens.iter_mut().skip(1).zip(amounts) {
                output_token.amount = amount;
            }
        }
    }
    Ok(SlpParseData {
        output_tokens,
        slp_token_type,
        slp_tx_type: parsed_opreturn.slp_tx_type,
        token_id,
    })
}

fn parse_opreturn_ops(ops: impl Iterator<Item = Op>) -> Result<Vec<Bytes>, SlpError> {
    let mut pushes = Vec::new();
    for (op_idx, op) in ops.into_iter().enumerate() {
        // first opcode must be OP_RETURN
        match (op_idx, &op) {
            (0, Op::Code(0x6a)) => continue,
            (0, &Op::Code(opcode)) | (0, &Op::Push(opcode, _)) => {
                return Err(SlpError::MissingOpReturn { opcode })
            }
            _ => {}
        }
        match op {
            Op::Code(opcode) => {
                if opcode == 0 || (0x4f..=0x60).contains(&opcode) {
                    return Err(SlpError::DisallowedPush { op_idx, opcode });
                }
                return Err(SlpError::NonPushOp { op_idx, opcode });
            }
            Op::Push(opcode, push) => {
                if opcode == 0 || opcode > 0x4e {
                    return Err(SlpError::DisallowedPush { op_idx, opcode });
                }
                pushes.push(push);
            }
        }
    }
    Ok(pushes)
}

fn parse_lokad_id(bytes: &Bytes) -> Result<(), SlpError> {
    if bytes.as_ref() != SLP_LOKAD_ID {
        return Err(SlpError::InvalidLokadId(bytes.clone()));
    }
    Ok(())
}

fn parse_token_type(bytes: &Bytes) -> Option<SlpTokenType> {
    if bytes.as_ref() == SLP_TOKEN_TYPE_V1 {
        Some(SlpTokenType::Fungible)
    } else if bytes.as_ref() == SLP_TOKEN_TYPE_V1_NFT1_GROUP {
        Some(SlpTokenType::Nft1Group)
    } else if bytes.as_ref() == SLP_TOKEN_TYPE_V1_NFT1_CHILD {
        Some(SlpTokenType::Nft1Child)
    } else {
        None
    }
}

struct ParsedOpReturn {
    slp_tx_type: SlpTxType,
    outputs: ParsedOutputs,
    token_id: Option<TokenId>,
}

enum ParsedOutputs {
    MintTokens {
        baton_out_idx: Option<usize>,
        mint_quantity: SlpAmount,
    },
    Send(Vec<SlpAmount>),
}

fn parse_genesis_data(
    opreturn_data: Vec<Bytes>,
    slp_token_type: SlpTokenType,
) -> Result<ParsedOpReturn, SlpError> {
    if opreturn_data.len() < 10 {
        return Err(SlpError::TooFewPushesExact {
            expected: 10,
            actual: opreturn_data.len(),
        });
    }
    if opreturn_data.len() > 10 {
        return Err(SlpError::SuperfluousPushes {
            expected: 10,
            actual: opreturn_data.len(),
        });
    }
    let mut data_iter = opreturn_data.into_iter();
    let _lokad_id = data_iter.next().unwrap();
    let _token_type = data_iter.next().unwrap();
    let _tx_type = data_iter.next().unwrap();
    let token_ticker = data_iter.next().unwrap();
    let token_name = data_iter.next().unwrap();
    let token_document_url = data_iter.next().unwrap();
    let token_document_hash = data_iter.next().unwrap();
    let decimals = data_iter.next().unwrap();
    let mint_baton_out_idx = data_iter.next().unwrap();
    let initial_quantity = data_iter.next().unwrap();
    assert!(data_iter.next().is_none());
    if token_document_hash.len() != 0 && token_document_hash.len() != 32 {
        return Err(SlpError::InvalidFieldSize {
            field_name: "token_document_hash",
            expected: &[0, 32],
            actual: token_document_hash.len(),
        });
    }
    if decimals.len() != 1 {
        return Err(SlpError::InvalidFieldSize {
            field_name: "decimals",
            expected: &[1],
            actual: decimals.len(),
        });
    }
    if mint_baton_out_idx.len() != 0 && mint_baton_out_idx.len() != 1 {
        return Err(SlpError::InvalidFieldSize {
            field_name: "mint_baton_out_idx",
            expected: &[0, 1],
            actual: mint_baton_out_idx.len(),
        });
    }
    let initial_quantity = SlpAmount::from_u64_be(&initial_quantity, "initial_quantity")?;
    if decimals[0] > 9 {
        return Err(SlpError::InvalidDecimals {
            actual: decimals[0] as usize,
        });
    }
    if mint_baton_out_idx.len() == 1 && mint_baton_out_idx[0] < 2 {
        return Err(SlpError::InvalidMintBatonIdx {
            actual: mint_baton_out_idx[0] as usize,
        });
    }
    let decimals = decimals[0] as u32;
    if slp_token_type == SlpTokenType::Nft1Child {
        if !mint_baton_out_idx.is_empty() {
            return Err(SlpError::Nft1ChildCannotHaveMintBaton);
        }
        if initial_quantity != SlpAmount::new(1) {
            return Err(SlpError::Nft1ChildInvalidInitialQuantity {
                actual: initial_quantity,
            });
        }
        if decimals != 0 {
            return Err(SlpError::Nft1ChildInvalidDecimals { actual: decimals });
        }
    }
    Ok(ParsedOpReturn {
        slp_tx_type: SlpTxType::Genesis(Box::new(SlpGenesisInfo {
            token_ticker,
            token_name,
            token_document_url,
            token_document_hash: ByteArray::from_slice(&token_document_hash).ok(),
            decimals,
        })),
        outputs: ParsedOutputs::MintTokens {
            baton_out_idx: mint_baton_out_idx
                .get(0)
                .map(|&mint_baton_out_idx| mint_baton_out_idx as usize),
            mint_quantity: initial_quantity,
        },
        token_id: None,
    })
}

fn parse_mint_data(opreturn_data: Vec<Bytes>) -> Result<ParsedOpReturn, SlpError> {
    if opreturn_data.len() < 6 {
        return Err(SlpError::TooFewPushesExact {
            expected: 6,
            actual: opreturn_data.len(),
        });
    }
    if opreturn_data.len() > 6 {
        return Err(SlpError::SuperfluousPushes {
            expected: 6,
            actual: opreturn_data.len(),
        });
    }
    let mut data_iter = opreturn_data.into_iter();
    let _lokad_id = data_iter.next().unwrap();
    let _token_type = data_iter.next().unwrap();
    let _tx_type = data_iter.next().unwrap();
    let token_id = data_iter.next().unwrap();
    let mint_baton_out_idx = data_iter.next().unwrap();
    let additional_quantity = data_iter.next().unwrap();
    assert!(data_iter.next().is_none());
    if token_id.len() != 32 {
        return Err(SlpError::InvalidFieldSize {
            field_name: "token_id",
            expected: &[32],
            actual: token_id.len(),
        });
    }
    if !(0..=1).contains(&mint_baton_out_idx.len()) {
        return Err(SlpError::InvalidFieldSize {
            field_name: "mint_baton_out_idx",
            expected: &[0, 1],
            actual: mint_baton_out_idx.len(),
        });
    }
    if mint_baton_out_idx.len() == 1 && mint_baton_out_idx[0] < 2 {
        return Err(SlpError::InvalidMintBatonIdx {
            actual: mint_baton_out_idx[0] as usize,
        });
    }
    let additional_quantity = SlpAmount::from_u64_be(&additional_quantity, "additional_quantity")?;
    Ok(ParsedOpReturn {
        slp_tx_type: SlpTxType::Mint,
        outputs: ParsedOutputs::MintTokens {
            baton_out_idx: mint_baton_out_idx
                .get(0)
                .map(|&mint_baton_out_idx| mint_baton_out_idx as usize),
            mint_quantity: additional_quantity,
        },
        token_id: Some(TokenId::from_slice_be(&token_id).unwrap()),
    })
}

fn parse_send_data(opreturn_data: Vec<Bytes>) -> Result<ParsedOpReturn, SlpError> {
    if opreturn_data.len() < 5 {
        return Err(SlpError::TooFewPushes {
            expected: 5,
            actual: opreturn_data.len(),
        });
    }
    if opreturn_data.len() > 23 {
        return Err(SlpError::SuperfluousPushes {
            expected: 23,
            actual: opreturn_data.len(),
        });
    }
    let mut data_iter = opreturn_data.into_iter();
    let _lokad_id = data_iter.next().unwrap();
    let _token_type = data_iter.next().unwrap();
    let _tx_type = data_iter.next().unwrap();
    let token_id = data_iter.next().unwrap();
    let output_quantities = data_iter;
    if token_id.len() != 32 {
        return Err(SlpError::InvalidFieldSize {
            field_name: "token_id",
            expected: &[32],
            actual: token_id.len(),
        });
    }
    let output_quantities = output_quantities
        .enumerate()
        .map(|(idx, quantity)| {
            SlpAmount::from_u64_be(&quantity, SLP_OUTPUT_QUANTITY_FIELD_NAMES[idx])
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ParsedOpReturn {
        slp_tx_type: SlpTxType::Send,
        outputs: ParsedOutputs::Send(output_quantities),
        token_id: Some(TokenId::from_slice_be(&token_id).unwrap()),
    })
}

#[cfg(test)]
mod tests {
    use bitcoinsuite_core::{BytesError, Script, Sha256d, TxOutput, UnhashedTx};
    use bitcoinsuite_error::Result;
    use pretty_assertions::assert_eq;

    use crate::{
        consts::SLP_OUTPUT_QUANTITY_FIELD_NAMES, parse_slp_tx, SlpAmount, SlpError, SlpGenesisInfo,
        SlpParseData, SlpToken, SlpTokenType, SlpTxType, TokenId,
    };

    #[test]
    fn test_parse_slp_tx() -> Result<()> {
        fn check_script(script: &[u8], expected_err: SlpError) {
            assert_eq!(
                parse_slp_tx(
                    &Sha256d::default(),
                    &UnhashedTx {
                        outputs: vec![TxOutput {
                            value: 0,
                            script: Script::from_slice(script),
                        }],
                        ..Default::default()
                    }
                ),
                Err(expected_err),
            );
        }
        let txid = Sha256d::new([4; 32]);
        // No outputs
        assert_eq!(
            parse_slp_tx(&txid, &UnhashedTx::default()),
            Err(SlpError::NoOutputs),
        );
        // Invalid OP_RETURN script
        check_script(
            &[0x01],
            SlpError::BytesError(BytesError::InvalidSplit {
                split_idx: 1,
                len: 0,
            }),
        );
        // Missing OP_RETURN opcode
        check_script(&[0xac], SlpError::MissingOpReturn { opcode: 0xac });
        // Disallowed push
        let mut scripts: Vec<(&[_], u8, usize)> = vec![
            (&[0x6a, 0x00], 0x00, 1),
            (&[0x6a, 0x4f], 0x4f, 1),
            (&[0x6a, 0x4c, 0x00, 0x51], 0x51, 2),
            (&[0x6a, 0x4d, 0x00, 0x00, 0x52], 0x52, 2),
            (&[0x6a, 0x4e, 0x00, 0x00, 0x00, 0x00, 0x53], 0x53, 2),
            (&[0x6a, 0x01, 0x00, 0x54], 0x54, 2),
            (&[0x6a, 0x02, 0x00, 0x00, 0x55], 0x55, 2),
            (&[0x6a, 0x56], 0x56, 1),
            (&[0x6a, 0x57], 0x57, 1),
            (&[0x6a, 0x58], 0x58, 1),
            (&[0x6a, 0x59], 0x59, 1),
            (&[0x6a, 0x5a], 0x5a, 1),
            (&[0x6a, 0x5b], 0x5b, 1),
            (&[0x6a, 0x5c], 0x5c, 1),
            (&[0x6a, 0x5d], 0x5d, 1),
            (&[0x6a, 0x5e], 0x5e, 1),
            (&[0x6a, 0x5f], 0x5f, 1),
            (&[0x6a, 0x60], 0x60, 1),
        ];
        let script = [[0x6a, 0x4b].as_ref(), &[0x00; 0x4b], &[0x00]].concat();
        scripts.push((&script, 0x00, 2));
        for (script, opcode, op_idx) in scripts {
            check_script(script, SlpError::DisallowedPush { opcode, op_idx });
        }
        // Non-pushop
        for opcode in 0x61..=0xff {
            check_script(&[0x6a, opcode], SlpError::NonPushOp { opcode, op_idx: 1 });
        }
        // Too few pushes
        let scripts = [
            [0x6a].as_ref(),
            &[0x6a, 0x01, 0x00],
            &[0x6a, 0x01, 0x00, 0x01, 0x00],
        ];
        for (num_pushes, script) in scripts.into_iter().enumerate() {
            check_script(
                script,
                SlpError::TooFewPushes {
                    expected: 3,
                    actual: num_pushes,
                },
            );
        }
        // Invalid LOKAD ID
        check_script(
            &[0x6a, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00],
            SlpError::InvalidLokadId([0x00].into()),
        );
        check_script(
            &[0x6a, 0x03, b'S', b'L', b'P', 0x01, 0x00, 0x01, 0x00],
            SlpError::InvalidLokadId(b"SLP".as_ref().into()),
        );
        check_script(
            &[0x6a, 0x04, b'S', b'L', b'P', 0x99, 0x01, 0x00, 0x01, 0x00],
            SlpError::InvalidLokadId(b"SLP\x99".as_ref().into()),
        );
        // Valid Lokad ID (using OP_PUSHDATA1, OP_PUSHDATA2 and OP_PUSHDATA4)
        check_script(
            &[
                0x6a, 0x4c, 0x04, b'S', b'L', b'P', 0x00, 0x4c, 0x00, 0x01, 0x00,
            ],
            SlpError::InvalidTokenType([].into()),
        );
        check_script(
            &[
                0x6a, 0x4d, 0x04, 0x00, b'S', b'L', b'P', 0x00, 0x4c, 0x00, 0x01, 0x00,
            ],
            SlpError::InvalidTokenType([].into()),
        );
        check_script(
            &[
                0x6a, 0x4e, 0x04, 0x00, 0x00, 0x00, b'S', b'L', b'P', 0x00, 0x4c, 0x00, 0x01, 0x00,
            ],
            SlpError::InvalidTokenType([].into()),
        );
        // Invalid token type
        check_script(
            &[0x6a, 0x04, b'S', b'L', b'P', 0x00, 0x4c, 0x00, 0x01, 0x00],
            SlpError::InvalidTokenType([].into()),
        );
        check_script(
            &[
                0x6a, 0x04, b'S', b'L', b'P', 0x00, 0x03, 0x99, 0x99, 0x99, 0x01, 0x00,
            ],
            SlpError::InvalidTokenType([0x99, 0x99, 0x99].into()),
        );
        // Unknown token type (no error, but results in "Unknown" fields)
        assert_eq!(
            parse_slp_tx(
                &Sha256d::default(),
                &UnhashedTx {
                    outputs: vec![TxOutput {
                        value: 0,
                        script: Script::from_slice(&[
                            0x6a, 0x04, b'S', b'L', b'P', 0x00, 0x02, 0x99, 0x99, 0x01, 0x00
                        ]),
                    }],
                    ..Default::default()
                }
            ),
            Ok(SlpParseData {
                output_tokens: vec![SlpToken::EMPTY],
                slp_token_type: SlpTokenType::Unknown,
                slp_tx_type: SlpTxType::Unknown,
                token_id: TokenId::new(Sha256d::new([0; 32])),
            }),
        );
        // Invalid tx type
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x07],
                b"INVALID",
            ]
            .concat(),
            SlpError::InvalidTxType(b"INVALID".as_ref().into()),
        );
        // Invalid GENESIS
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x07],
                b"GENESIS",
            ]
            .concat(),
            SlpError::TooFewPushesExact {
                expected: 10,
                actual: 3,
            },
        );
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x07],
                b"GENESIS",
                &[0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00],
                &[0x01, 0x00, 0x01, 0x00, 0x01, 0x00],
            ]
            .concat(),
            SlpError::SuperfluousPushes {
                expected: 10,
                actual: 11,
            },
        );
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x07],
                b"GENESIS",
                &[0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00],
                &[0x01, 0x00, 0x01, 0x00],
            ]
            .concat(),
            SlpError::InvalidFieldSize {
                field_name: "token_document_hash",
                actual: 1,
                expected: &[0, 32],
            },
        );
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x07],
                b"GENESIS",
                &[0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x4c, 0x00],
                &[0x4c, 0x00],
                &[0x01, 0x00, 0x01, 0x00],
            ]
            .concat(),
            SlpError::InvalidFieldSize {
                field_name: "decimals",
                actual: 0,
                expected: &[1],
            },
        );
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x07],
                b"GENESIS",
                &[0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x4c, 0x00],
                &[0x02, 0x00, 0x00],
                &[0x01, 0x00, 0x01, 0x00],
            ]
            .concat(),
            SlpError::InvalidFieldSize {
                field_name: "decimals",
                actual: 2,
                expected: &[1],
            },
        );
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x07],
                b"GENESIS",
                &[0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x4c, 0x00],
                &[0x01, 0x00],
                &[0x02, 0x00, 0x00],
                &[0x01, 0x00],
            ]
            .concat(),
            SlpError::InvalidFieldSize {
                field_name: "mint_baton_out_idx",
                actual: 2,
                expected: &[0, 1],
            },
        );
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x07],
                b"GENESIS",
                &[0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x4c, 0x00],
                &[0x01, 0x00],
                &[0x01, 0x00],
                &[0x01, 0x00],
            ]
            .concat(),
            SlpError::InvalidFieldSize {
                field_name: "initial_quantity",
                actual: 1,
                expected: &[8],
            },
        );
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x07],
                b"GENESIS",
                &[0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x4c, 0x00],
                &[0x01, 10],
                &[0x01, 0x00],
                &[0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            ]
            .concat(),
            SlpError::InvalidDecimals { actual: 10 },
        );
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x07],
                b"GENESIS",
                &[0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x4c, 0x00],
                &[0x01, 0x09],
                &[0x01, 0x01],
                &[0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            ]
            .concat(),
            SlpError::InvalidMintBatonIdx { actual: 0x01 },
        );
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 0x41],
                &[0x07],
                b"GENESIS",
                &[0x01, 0x44, 0x01, 0x55, 0x01, 0x66, 0x4c, 0x00],
                &[0x01, 0x09],
                &[0x01, 0x02],
                &[0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            ]
            .concat(),
            SlpError::Nft1ChildCannotHaveMintBaton,
        );
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 0x41],
                &[0x07],
                b"GENESIS",
                &[0x01, 0x44, 0x01, 0x55, 0x01, 0x66, 0x4c, 0x00],
                &[0x01, 0x09],
                &[0x4c, 0x00],
                &[0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 123],
            ]
            .concat(),
            SlpError::Nft1ChildInvalidInitialQuantity {
                actual: SlpAmount::new(123),
            },
        );
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 0x41],
                &[0x07],
                b"GENESIS",
                &[0x01, 0x44, 0x01, 0x55, 0x01, 0x66, 0x4c, 0x00],
                &[0x01, 0x09],
                &[0x4c, 0x00],
                &[0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01],
            ]
            .concat(),
            SlpError::Nft1ChildInvalidDecimals { actual: 9 },
        );
        // Valid GENESIS
        assert_eq!(
            parse_slp_tx(
                &Sha256d::new([3; 32]),
                &UnhashedTx {
                    outputs: vec![
                        TxOutput {
                            value: 0,
                            script: Script::from_slice(
                                &[
                                    [0x6a, 0x04].as_ref(),
                                    b"SLP\0",
                                    &[0x01, 1],
                                    &[0x07],
                                    b"GENESIS",
                                    &[0x01, 0x44, 0x01, 0x55, 0x01, 0x66, 0x4c, 0x00],
                                    &[0x01, 0x09],
                                    &[0x01, 0x02],
                                    &[0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 123],
                                ]
                                .concat()
                            ),
                        },
                        TxOutput::default(),
                        TxOutput::default(),
                    ],
                    ..Default::default()
                }
            ),
            Ok(SlpParseData {
                output_tokens: vec![SlpToken::EMPTY, SlpToken::amount(123), SlpToken::MINT_BATON],
                slp_token_type: SlpTokenType::Fungible,
                slp_tx_type: SlpTxType::Genesis(Box::new(SlpGenesisInfo {
                    token_ticker: [0x44].into(),
                    token_name: [0x55].into(),
                    token_document_url: [0x66].into(),
                    token_document_hash: None,
                    decimals: 9
                })),
                token_id: TokenId::new(Sha256d::new([3; 32])),
            }),
        );
        for (type_byte, token_type) in [
            (1, SlpTokenType::Fungible),
            (0x41, SlpTokenType::Nft1Child),
            (0x81, SlpTokenType::Nft1Group),
        ] {
            let qty = match token_type {
                SlpTokenType::Nft1Child => 1,
                _ => 123,
            };
            assert_eq!(
                parse_slp_tx(
                    &Sha256d::new([3; 32]),
                    &UnhashedTx {
                        outputs: vec![
                            TxOutput {
                                value: 0,
                                script: Script::from_slice(
                                    &[
                                        [0x6a, 0x04].as_ref(),
                                        b"SLP\0",
                                        &[0x01, type_byte],
                                        &[0x07],
                                        b"GENESIS",
                                        &[0x01, 0x44, 0x01, 0x55, 0x01, 0x66, 0x4c, 0x00],
                                        match token_type {
                                            SlpTokenType::Nft1Child => &[0x01, 0x00],
                                            _ => &[0x01, 0x09],
                                        },
                                        match token_type {
                                            SlpTokenType::Nft1Child => &[0x4c, 0x00],
                                            _ => &[0x01, 0x02],
                                        },
                                        &[0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, qty],
                                    ]
                                    .concat()
                                ),
                            },
                            TxOutput::default(),
                            TxOutput::default(),
                        ],
                        ..Default::default()
                    }
                ),
                Ok(SlpParseData {
                    output_tokens: vec![
                        SlpToken::EMPTY,
                        SlpToken::amount(qty as i128),
                        match token_type {
                            SlpTokenType::Nft1Child => SlpToken::EMPTY,
                            _ => SlpToken::MINT_BATON,
                        },
                    ],
                    slp_token_type: token_type,
                    slp_tx_type: SlpTxType::Genesis(Box::new(SlpGenesisInfo {
                        token_ticker: [0x44].into(),
                        token_name: [0x55].into(),
                        token_document_url: [0x66].into(),
                        token_document_hash: None,
                        decimals: match token_type {
                            SlpTokenType::Nft1Child => 0,
                            _ => 9,
                        },
                    })),
                    token_id: TokenId::new(Sha256d::new([3; 32])),
                }),
            );
        }
        // Invalid MINT
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x04],
                b"MINT",
            ]
            .concat(),
            SlpError::TooFewPushesExact {
                expected: 6,
                actual: 3,
            },
        );
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x04],
                b"MINT",
                &[0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00],
            ]
            .concat(),
            SlpError::SuperfluousPushes {
                expected: 6,
                actual: 7,
            },
        );
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x04],
                b"MINT",
                &[0x01, 0x00, 0x01, 0x00, 0x01, 0x00],
            ]
            .concat(),
            SlpError::InvalidFieldSize {
                field_name: "token_id",
                actual: 1,
                expected: &[32],
            },
        );
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x04],
                b"MINT",
                &[0x20],
                &[0x44; 32],
                &[0x02, 0x00, 0x00],
                &[0x01, 0x00],
            ]
            .concat(),
            SlpError::InvalidFieldSize {
                field_name: "mint_baton_out_idx",
                actual: 2,
                expected: &[0, 1],
            },
        );
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x04],
                b"MINT",
                &[0x20],
                &[0x44; 32],
                &[0x01, 0x01],
                &[0x01, 0x00],
            ]
            .concat(),
            SlpError::InvalidMintBatonIdx { actual: 1 },
        );
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x04],
                b"MINT",
                &[0x20],
                &[0x44; 32],
                &[0x01, 0x02],
                &[0x01, 0x00],
            ]
            .concat(),
            SlpError::InvalidFieldSize {
                field_name: "additional_quantity",
                actual: 1,
                expected: &[8],
            },
        );
        // Valid MINT
        for (type_byte, token_type) in [
            (1, SlpTokenType::Fungible),
            (0x41, SlpTokenType::Nft1Child),
            (0x81, SlpTokenType::Nft1Group),
        ] {
            assert_eq!(
                parse_slp_tx(
                    &Sha256d::new([3; 32]),
                    &UnhashedTx {
                        outputs: vec![
                            TxOutput {
                                value: 0,
                                script: Script::from_slice(
                                    &[
                                        [0x6a, 0x04].as_ref(),
                                        b"SLP\0",
                                        &[0x01, type_byte],
                                        &[0x04],
                                        b"MINT",
                                        &[0x20],
                                        &[0x44; 32],
                                        &[0x01, 0x02],
                                        &[0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 231],
                                    ]
                                    .concat()
                                ),
                            },
                            TxOutput::default(),
                            TxOutput::default(),
                        ],
                        ..Default::default()
                    }
                ),
                Ok(SlpParseData {
                    output_tokens: vec![
                        SlpToken::EMPTY,
                        SlpToken::amount(231),
                        SlpToken::MINT_BATON,
                    ],
                    slp_token_type: token_type,
                    slp_tx_type: SlpTxType::Mint,
                    token_id: TokenId::new(Sha256d::new([0x44; 32])),
                }),
            );
        }
        // Invalid SEND
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x04],
                b"SEND",
            ]
            .concat(),
            SlpError::TooFewPushes {
                expected: 5,
                actual: 3,
            },
        );
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x04],
                b"SEND",
                &[0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00],
                &[0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00],
                &[0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00],
                &[0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00],
                &[0x01, 0x00],
            ]
            .concat(),
            SlpError::SuperfluousPushes {
                expected: 23,
                actual: 24,
            },
        );
        check_script(
            &[
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x04],
                b"SEND",
                &[0x01, 0x00, 0x01, 0x00],
            ]
            .concat(),
            SlpError::InvalidFieldSize {
                field_name: "token_id",
                expected: &[32],
                actual: 1,
            },
        );
        // Test all possible SEND outputs with one amount having 2 bytes
        for num_outputs in 1..=19 {
            let script_intro = [
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, 1],
                &[0x04],
                b"SEND",
                &[0x20],
                &[0x22; 32],
            ]
            .concat();
            for (invalid_idx, field_name) in SLP_OUTPUT_QUANTITY_FIELD_NAMES
                .iter()
                .enumerate()
                .take(num_outputs)
            {
                let mut script = script_intro.clone();
                for idx in 0..num_outputs {
                    if invalid_idx == idx {
                        script.extend([0x02, 0x00, 0x00]);
                    } else {
                        script.extend([0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
                    }
                }
                check_script(
                    &script,
                    SlpError::InvalidFieldSize {
                        field_name,
                        expected: &[8],
                        actual: 2,
                    },
                );
            }
        }
        // Valid SEND
        for (type_byte, token_type) in [
            (1, SlpTokenType::Fungible),
            (0x41, SlpTokenType::Nft1Child),
            (0x81, SlpTokenType::Nft1Group),
        ] {
            let script_intro = [
                [0x6a, 0x04].as_ref(),
                b"SLP\0",
                &[0x01, type_byte],
                &[0x04],
                b"SEND",
                &[0x20],
                &[0x22; 32],
            ]
            .concat();
            for num_amounts in 1..=19 {
                let mut script = script_intro.clone();
                let mut amounts = vec![SlpToken::EMPTY];
                for idx in 1..=num_amounts {
                    script.extend([0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, idx as u8]);
                    amounts.push(SlpToken::amount(idx as i128));
                }
                // output_tokens is independent of tx.outputs
                for num_tx_outputs in 1..=20 {
                    let mut tx_outputs = vec![TxOutput::default(); num_tx_outputs];
                    tx_outputs[0].script = Script::from_slice(&script);
                    assert_eq!(
                        parse_slp_tx(
                            &Sha256d::new([3; 32]),
                            &UnhashedTx {
                                outputs: tx_outputs,
                                ..Default::default()
                            },
                        ),
                        Ok(SlpParseData {
                            output_tokens: amounts.clone(),
                            slp_token_type: token_type,
                            slp_tx_type: SlpTxType::Send,
                            token_id: TokenId::new(Sha256d::new([0x22; 32])),
                        }),
                    );
                }
            }
        }
        Ok(())
    }
}
