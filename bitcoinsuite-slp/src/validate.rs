use crate::{
    SlpAmount, SlpBurn, SlpError, SlpParseData, SlpToken, SlpTokenType, SlpTxData, SlpTxType,
    TokenId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlpValidTxData {
    pub slp_tx_data: SlpTxData,
    pub slp_burns: Vec<Option<Box<SlpBurn>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlpSpentOutput {
    pub token_id: TokenId,
    pub token_type: SlpTokenType,
    pub token: SlpToken,
    pub group_token_id: Option<Box<TokenId>>,
}

pub fn validate_slp_tx(
    parse_data: SlpParseData,
    spent_outputs: &[Option<SlpSpentOutput>],
) -> Result<SlpValidTxData, SlpError> {
    let mut input_tokens = Vec::with_capacity(spent_outputs.len());
    let mut slp_burns = Vec::with_capacity(spent_outputs.len());
    let mut group_token_id = None;
    match &parse_data.slp_tx_type {
        SlpTxType::Genesis(_) => {
            for spent_output in spent_outputs {
                input_tokens.push(SlpToken::EMPTY);
                match spent_output {
                    Some(spent_output) => {
                        slp_burns.push(Some(Box::new(SlpBurn {
                            token: spent_output.token,
                            token_id: spent_output.token_id.clone(),
                        })));
                    }
                    None => slp_burns.push(None),
                }
            }
            if parse_data.slp_token_type == SlpTokenType::Nft1Child {
                let spent_output = spent_outputs
                    .get(0)
                    .and_then(|x| x.as_ref())
                    .ok_or(SlpError::HasNoNft1Group)?;
                if spent_output.token_type != SlpTokenType::Nft1Group
                    || spent_output.token.amount == SlpAmount::default()
                {
                    return Err(SlpError::HasNoNft1Group);
                }
                input_tokens[0] = spent_output.token;
                slp_burns[0] = None;
                group_token_id = Some(Box::new(spent_output.token_id.clone()));
            }
        }
        SlpTxType::Mint => {
            let mut has_mint_baton = false;
            for spent_output in spent_outputs {
                match spent_output {
                    Some(spent_output) => {
                        if parse_data.token_id == spent_output.token_id
                            && parse_data.slp_token_type == spent_output.token_type
                            && spent_output.token.is_mint_baton
                        {
                            // Found mint baton
                            has_mint_baton = true;
                            slp_burns.push(None);
                            input_tokens.push(spent_output.token);
                        } else {
                            // Invalid SLP input, burn it
                            slp_burns.push(Some(Box::new(SlpBurn {
                                token: spent_output.token,
                                token_id: spent_output.token_id.clone(),
                            })));
                            input_tokens.push(SlpToken::EMPTY);
                        }
                    }
                    None => {
                        slp_burns.push(None);
                        input_tokens.push(SlpToken::EMPTY);
                    }
                }
            }
            if !has_mint_baton {
                return Err(SlpError::HasNoMintBaton);
            }
        }
        SlpTxType::Send => {
            let output_sum = parse_data
                .output_tokens
                .iter()
                .map(|token| token.amount)
                .sum::<SlpAmount>();
            let mut input_sum = SlpAmount::new(0);
            for spent_output in spent_outputs {
                match spent_output {
                    Some(spent_output) => {
                        if parse_data.token_id == spent_output.token_id
                            && parse_data.slp_token_type == spent_output.token_type
                            && !spent_output.token.is_mint_baton
                        {
                            // Valid input which is not a mint_baton
                            input_tokens.push(spent_output.token);
                            input_sum += spent_output.token.amount;
                            if group_token_id.is_none() {
                                group_token_id = spent_output.group_token_id.clone();
                            }
                            if input_sum > output_sum {
                                let total_burned = input_sum - output_sum;
                                let burned_amount = if total_burned < spent_output.token.amount {
                                    total_burned
                                } else {
                                    spent_output.token.amount
                                };
                                slp_burns.push(Some(Box::new(SlpBurn {
                                    token: SlpToken {
                                        amount: burned_amount,
                                        is_mint_baton: false,
                                    },
                                    token_id: spent_output.token_id.clone(),
                                })));
                            } else {
                                slp_burns.push(None);
                            }
                        } else {
                            // Invalid SLP input, burn it
                            slp_burns.push(Some(Box::new(SlpBurn {
                                token: spent_output.token,
                                token_id: spent_output.token_id.clone(),
                            })));
                            input_tokens.push(SlpToken::EMPTY);
                        }
                    }
                    None => {
                        slp_burns.push(None);
                        input_tokens.push(SlpToken::EMPTY);
                    }
                }
            }
            if output_sum > input_sum {
                return Err(SlpError::OutputSumExceedInputSum {
                    output_sum,
                    input_sum,
                });
            }
        }
        SlpTxType::Unknown => {
            for spent_output in spent_outputs {
                input_tokens.push(SlpToken::EMPTY);
                match spent_output {
                    Some(spent_output) => {
                        slp_burns.push(Some(Box::new(SlpBurn {
                            token: spent_output.token,
                            token_id: spent_output.token_id.clone(),
                        })));
                    }
                    None => slp_burns.push(None),
                }
            }
        }
    }
    Ok(SlpValidTxData {
        slp_tx_data: SlpTxData {
            input_tokens,
            output_tokens: parse_data.output_tokens,
            slp_token_type: parse_data.slp_token_type,
            slp_tx_type: parse_data.slp_tx_type,
            token_id: parse_data.token_id,
            group_token_id,
        },
        slp_burns,
    })
}

#[cfg(test)]
mod tests {
    use bitcoinsuite_core::Sha256d;
    use bitcoinsuite_error::Result;
    use pretty_assertions::assert_eq;

    use crate::{
        validate_slp_tx, SlpAmount, SlpBurn, SlpError, SlpGenesisInfo, SlpParseData,
        SlpSpentOutput, SlpToken, SlpTokenType, SlpTxData, SlpTxType, SlpValidTxData, TokenId,
    };

    #[test]
    fn test_validate_slp_tx_genesis_failure() -> Result<()> {
        // Missing NFT1 Group token
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Nft1Child,
                    slp_tx_type: SlpTxType::Genesis(Box::new(SlpGenesisInfo::default())),
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[None],
            ),
            Err(SlpError::HasNoNft1Group),
        );
        // Invalid NFT1 Group token amount and token type
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Nft1Child,
                    slp_tx_type: SlpTxType::Genesis(Box::new(SlpGenesisInfo::default())),
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[Some(SlpSpentOutput {
                    token_id: TokenId::new(Sha256d::new([3; 32])),
                    token_type: SlpTokenType::Fungible,
                    token: SlpToken::EMPTY,
                    group_token_id: None,
                })],
            ),
            Err(SlpError::HasNoNft1Group),
        );
        // Invalid NFT1 Group token amount
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Nft1Child,
                    slp_tx_type: SlpTxType::Genesis(Box::new(SlpGenesisInfo::default())),
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[Some(SlpSpentOutput {
                    token_id: TokenId::new(Sha256d::new([3; 32])),
                    token_type: SlpTokenType::Nft1Group,
                    token: SlpToken::EMPTY,
                    group_token_id: None,
                })],
            ),
            Err(SlpError::HasNoNft1Group),
        );
        // Invalid NFT1 Group token type
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Nft1Child,
                    slp_tx_type: SlpTxType::Genesis(Box::new(SlpGenesisInfo::default())),
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[Some(SlpSpentOutput {
                    token_id: TokenId::new(Sha256d::new([3; 32])),
                    token_type: SlpTokenType::Fungible,
                    token: SlpToken::amount(1),
                    group_token_id: None,
                })],
            ),
            Err(SlpError::HasNoNft1Group),
        );
        // Invalid NFT1 Group token input index (must be at 0)
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Nft1Child,
                    slp_tx_type: SlpTxType::Genesis(Box::new(SlpGenesisInfo::default())),
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[
                    None,
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([3; 32])),
                        token_type: SlpTokenType::Nft1Child,
                        token: SlpToken::amount(1),
                        group_token_id: None,
                    })
                ],
            ),
            Err(SlpError::HasNoNft1Group),
        );
        Ok(())
    }

    #[test]
    fn test_validate_slp_tx_genesis_success() -> Result<()> {
        // Fungible token genesis
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Genesis(Default::default()),
                    token_id: TokenId::new(Sha256d::new([1; 32])),
                },
                &[None],
            ),
            Ok(SlpValidTxData {
                slp_tx_data: SlpTxData {
                    input_tokens: vec![SlpToken::EMPTY],
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Genesis(Default::default()),
                    token_id: TokenId::new(Sha256d::new([1; 32])),
                    group_token_id: None,
                },
                slp_burns: vec![None],
            }),
        );
        // Fungible genesis burning another token
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Genesis(Default::default()),
                    token_id: TokenId::new(Sha256d::new([2; 32])),
                },
                &[Some(SlpSpentOutput {
                    token_id: TokenId::new(Sha256d::new([1; 32])),
                    token_type: SlpTokenType::Fungible,
                    token: SlpToken::amount(1),
                    group_token_id: None,
                })],
            ),
            Ok(SlpValidTxData {
                slp_tx_data: SlpTxData {
                    input_tokens: vec![SlpToken::EMPTY],
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Genesis(Default::default()),
                    token_id: TokenId::new(Sha256d::new([2; 32])),
                    group_token_id: None,
                },
                slp_burns: vec![Some(Box::new(SlpBurn {
                    token: SlpToken::amount(1),
                    token_id: TokenId::new(Sha256d::new([1; 32])),
                }))],
            }),
        );
        // NFT1 Child genesis consuming NFT1 Group
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Nft1Child,
                    slp_tx_type: SlpTxType::Genesis(Default::default()),
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[Some(SlpSpentOutput {
                    token_id: TokenId::new(Sha256d::new([3; 32])),
                    token_type: SlpTokenType::Nft1Group,
                    token: SlpToken::amount(4),
                    group_token_id: None,
                })],
            ),
            Ok(SlpValidTxData {
                slp_tx_data: SlpTxData {
                    input_tokens: vec![SlpToken::amount(4)],
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Nft1Child,
                    slp_tx_type: SlpTxType::Genesis(Default::default()),
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                    group_token_id: Some(Box::new(TokenId::new(Sha256d::new([3; 32])))),
                },
                slp_burns: vec![None],
            }),
        );
        // NFT1 Child genesis consuming one NFT1 Group and burning another
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Nft1Child,
                    slp_tx_type: SlpTxType::Genesis(Default::default()),
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([3; 32])),
                        token_type: SlpTokenType::Nft1Group,
                        token: SlpToken::amount(4),
                        group_token_id: None,
                    }),
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([2; 32])),
                        token_type: SlpTokenType::Nft1Group,
                        token: SlpToken::amount(1),
                        group_token_id: None,
                    }),
                ],
            ),
            Ok(SlpValidTxData {
                slp_tx_data: SlpTxData {
                    input_tokens: vec![SlpToken::amount(4), SlpToken::EMPTY],
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Nft1Child,
                    slp_tx_type: SlpTxType::Genesis(Default::default()),
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                    group_token_id: Some(Box::new(TokenId::new(Sha256d::new([3; 32])))),
                },
                slp_burns: vec![
                    None,
                    Some(Box::new(SlpBurn {
                        token: SlpToken::amount(1),
                        token_id: TokenId::new(Sha256d::new([2; 32])),
                    })),
                ],
            }),
        );
        Ok(())
    }

    #[test]
    fn test_validate_slp_tx_mint_failure() -> Result<()> {
        // No SLP inputs
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Mint,
                    token_id: TokenId::new(Sha256d::new([1; 32])),
                },
                &[None],
            ),
            Err(SlpError::HasNoMintBaton),
        );
        // No MINT input
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Mint,
                    token_id: TokenId::new(Sha256d::new([1; 32])),
                },
                &[Some(SlpSpentOutput {
                    token_id: TokenId::new(Sha256d::new([1; 32])),
                    token_type: SlpTokenType::Fungible,
                    token: SlpToken::amount(4),
                    group_token_id: None,
                })],
            ),
            Err(SlpError::HasNoMintBaton),
        );
        // Wrong MINT input token ID
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Mint,
                    token_id: TokenId::new(Sha256d::new([1; 32])),
                },
                &[Some(SlpSpentOutput {
                    token_id: TokenId::new(Sha256d::new([2; 32])),
                    token_type: SlpTokenType::Fungible,
                    token: SlpToken::MINT_BATON,
                    group_token_id: None,
                })],
            ),
            Err(SlpError::HasNoMintBaton),
        );
        // Big Fungible example with lots of wrong MINT batons
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Mint,
                    token_id: TokenId::new(Sha256d::new([1; 32])),
                },
                &[
                    None,
                    // Not a MINT baton
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                        token_type: SlpTokenType::Fungible,
                        token: SlpToken::amount(4),
                        group_token_id: None,
                    }),
                    None,
                    // Wrong token ID
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([2; 32])),
                        token_type: SlpTokenType::Fungible,
                        token: SlpToken::MINT_BATON,
                        group_token_id: None,
                    }),
                    // Wrong token type (NFT1 Group)
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                        token_type: SlpTokenType::Nft1Group,
                        token: SlpToken::MINT_BATON,
                        group_token_id: None,
                    }),
                    // Wrong token type (NFT1 Child)
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                        token_type: SlpTokenType::Nft1Child,
                        token: SlpToken::MINT_BATON,
                        group_token_id: Some(Box::new(TokenId::new(Sha256d::new([10; 32])))),
                    }),
                    None,
                ],
            ),
            Err(SlpError::HasNoMintBaton),
        );
        // Big NFT1 Group example with lots of wrong batons
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Nft1Group,
                    slp_tx_type: SlpTxType::Mint,
                    token_id: TokenId::new(Sha256d::new([1; 32])),
                },
                &[
                    None,
                    // Not a MINT baton
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                        token_type: SlpTokenType::Nft1Group,
                        token: SlpToken::amount(4),
                        group_token_id: None,
                    }),
                    None,
                    // Wrong token ID
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([2; 32])),
                        token_type: SlpTokenType::Nft1Group,
                        token: SlpToken::MINT_BATON,
                        group_token_id: None,
                    }),
                    // Wrong token type (Fungible)
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                        token_type: SlpTokenType::Fungible,
                        token: SlpToken::MINT_BATON,
                        group_token_id: None,
                    }),
                    // Wrong token type (NFT1 Child)
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                        token_type: SlpTokenType::Nft1Child,
                        token: SlpToken::MINT_BATON,
                        group_token_id: Some(Box::new(TokenId::new(Sha256d::new([10; 32])))),
                    }),
                    None,
                ],
            ),
            Err(SlpError::HasNoMintBaton),
        );
        Ok(())
    }

    #[test]
    fn test_validate_slp_tx_mint_success() -> Result<()> {
        // Fungible MINT
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Mint,
                    token_id: TokenId::new(Sha256d::new([1; 32])),
                },
                &[Some(SlpSpentOutput {
                    token_id: TokenId::new(Sha256d::new([1; 32])),
                    token_type: SlpTokenType::Fungible,
                    token: SlpToken::MINT_BATON,
                    group_token_id: None,
                })],
            ),
            Ok(SlpValidTxData {
                slp_tx_data: SlpTxData {
                    input_tokens: vec![SlpToken::MINT_BATON],
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Mint,
                    token_id: TokenId::new(Sha256d::new([1; 32])),
                    group_token_id: None,
                },
                slp_burns: vec![None],
            }),
        );
        // Fungible MINT with lots of wrong batons and one correct one
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Mint,
                    token_id: TokenId::new(Sha256d::new([1; 32])),
                },
                &[
                    None,
                    // Not a MINT baton
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                        token_type: SlpTokenType::Fungible,
                        token: SlpToken::amount(4),
                        group_token_id: None,
                    }),
                    None,
                    // Wrong token ID
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([2; 32])),
                        token_type: SlpTokenType::Fungible,
                        token: SlpToken::MINT_BATON,
                        group_token_id: None,
                    }),
                    // Wrong token type (NFT1 Group)
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                        token_type: SlpTokenType::Nft1Group,
                        token: SlpToken::MINT_BATON,
                        group_token_id: None,
                    }),
                    // Correct MINT baton
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                        token_type: SlpTokenType::Fungible,
                        token: SlpToken::MINT_BATON,
                        group_token_id: None,
                    }),
                    // Wrong token type (NFT1 Child)
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                        token_type: SlpTokenType::Nft1Child,
                        token: SlpToken::MINT_BATON,
                        group_token_id: Some(Box::new(TokenId::new(Sha256d::new([10; 32])))),
                    }),
                    None,
                ],
            ),
            Ok(SlpValidTxData {
                slp_tx_data: SlpTxData {
                    input_tokens: vec![
                        SlpToken::EMPTY,
                        SlpToken::EMPTY,
                        SlpToken::EMPTY,
                        SlpToken::EMPTY,
                        SlpToken::EMPTY,
                        SlpToken::MINT_BATON,
                        SlpToken::EMPTY,
                        SlpToken::EMPTY,
                    ],
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Mint,
                    token_id: TokenId::new(Sha256d::new([1; 32])),
                    group_token_id: None,
                },
                slp_burns: vec![
                    None,
                    Some(Box::new(SlpBurn {
                        token: SlpToken::amount(4),
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                    })),
                    None,
                    Some(Box::new(SlpBurn {
                        token: SlpToken::MINT_BATON,
                        token_id: TokenId::new(Sha256d::new([2; 32])),
                    })),
                    Some(Box::new(SlpBurn {
                        token: SlpToken::MINT_BATON,
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                    })),
                    None, // Correct MINT baton not burned
                    Some(Box::new(SlpBurn {
                        token: SlpToken::MINT_BATON,
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                    })),
                    None,
                ],
            }),
        );
        // NFT Group MINT with lots of invalid batons and one correct one
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Nft1Group,
                    slp_tx_type: SlpTxType::Mint,
                    token_id: TokenId::new(Sha256d::new([1; 32])),
                },
                &[
                    None,
                    // Not a MINT baton
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                        token_type: SlpTokenType::Fungible,
                        token: SlpToken::amount(4),
                        group_token_id: None,
                    }),
                    None,
                    // Wrong token ID
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([2; 32])),
                        token_type: SlpTokenType::Fungible,
                        token: SlpToken::MINT_BATON,
                        group_token_id: None,
                    }),
                    // Correct MINT baton
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                        token_type: SlpTokenType::Nft1Group,
                        token: SlpToken::MINT_BATON,
                        group_token_id: None,
                    }),
                    // Wrong token type (Fungible)
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                        token_type: SlpTokenType::Fungible,
                        token: SlpToken::MINT_BATON,
                        group_token_id: None,
                    }),
                    // Wrong token type (NFT1 Child)
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                        token_type: SlpTokenType::Nft1Child,
                        token: SlpToken::MINT_BATON,
                        group_token_id: Some(Box::new(TokenId::new(Sha256d::new([10; 32])))),
                    }),
                    None,
                ],
            ),
            Ok(SlpValidTxData {
                slp_tx_data: SlpTxData {
                    input_tokens: vec![
                        SlpToken::EMPTY,
                        SlpToken::EMPTY,
                        SlpToken::EMPTY,
                        SlpToken::EMPTY,
                        SlpToken::MINT_BATON,
                        SlpToken::EMPTY,
                        SlpToken::EMPTY,
                        SlpToken::EMPTY,
                    ],
                    output_tokens: vec![],
                    slp_token_type: SlpTokenType::Nft1Group,
                    slp_tx_type: SlpTxType::Mint,
                    token_id: TokenId::new(Sha256d::new([1; 32])),
                    group_token_id: None,
                },
                slp_burns: vec![
                    None,
                    Some(Box::new(SlpBurn {
                        token: SlpToken::amount(4),
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                    })),
                    None,
                    Some(Box::new(SlpBurn {
                        token: SlpToken::MINT_BATON,
                        token_id: TokenId::new(Sha256d::new([2; 32])),
                    })),
                    None, // Correct MINT baton not burned
                    Some(Box::new(SlpBurn {
                        token: SlpToken::MINT_BATON,
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                    })),
                    Some(Box::new(SlpBurn {
                        token: SlpToken::MINT_BATON,
                        token_id: TokenId::new(Sha256d::new([1; 32])),
                    })),
                    None,
                ],
            }),
        );
        Ok(())
    }

    #[test]
    fn test_validate_slp_tx_send_failure() -> Result<()> {
        // No input tokens
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![SlpToken::amount(4)],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[None],
            ),
            Err(SlpError::OutputSumExceedInputSum {
                input_sum: SlpAmount::default(),
                output_sum: SlpAmount::new(4),
            }),
        );
        // Fungible inputs not enough (3 < 4)
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![SlpToken::amount(4)],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[Some(SlpSpentOutput {
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                    token_type: SlpTokenType::Fungible,
                    token: SlpToken::amount(3),
                    group_token_id: None,
                })],
            ),
            Err(SlpError::OutputSumExceedInputSum {
                input_sum: SlpAmount::new(3),
                output_sum: SlpAmount::new(4),
            }),
        );
        // Wrong input token type (expected Fungible, got NFT1 Child)
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![SlpToken::amount(4)],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[Some(SlpSpentOutput {
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                    token_type: SlpTokenType::Nft1Child,
                    token: SlpToken::amount(1),
                    group_token_id: Some(Box::new(TokenId::new(Sha256d::new([10; 32])))),
                })],
            ),
            Err(SlpError::OutputSumExceedInputSum {
                input_sum: SlpAmount::default(),
                output_sum: SlpAmount::new(4),
            }),
        );
        // NFT1 Group inputs not enough (3 < 4)
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![SlpToken::amount(4)],
                    slp_token_type: SlpTokenType::Nft1Group,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[Some(SlpSpentOutput {
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                    token_type: SlpTokenType::Nft1Group,
                    token: SlpToken::amount(3),
                    group_token_id: None,
                })],
            ),
            Err(SlpError::OutputSumExceedInputSum {
                input_sum: SlpAmount::new(3),
                output_sum: SlpAmount::new(4),
            }),
        );
        // Wrong input token type (expected NFT1 Group, got NFT1 Child)
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![SlpToken::amount(4)],
                    slp_token_type: SlpTokenType::Nft1Group,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[Some(SlpSpentOutput {
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                    token_type: SlpTokenType::Nft1Child,
                    token: SlpToken::amount(1),
                    group_token_id: Some(Box::new(TokenId::new(Sha256d::new([10; 32])))),
                })],
            ),
            Err(SlpError::OutputSumExceedInputSum {
                input_sum: SlpAmount::default(),
                output_sum: SlpAmount::new(4),
            }),
        );
        // Wrong input token ID
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![SlpToken::amount(4)],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[Some(SlpSpentOutput {
                    token_id: TokenId::new(Sha256d::new([3; 32])),
                    token_type: SlpTokenType::Fungible,
                    token: SlpToken::amount(5),
                    group_token_id: None,
                })],
            ),
            Err(SlpError::OutputSumExceedInputSum {
                input_sum: SlpAmount::default(),
                output_sum: SlpAmount::new(4),
            }),
        );
        // Big Fungible with off-by-one too little input tokens
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![
                        SlpToken::amount(1),
                        SlpToken::amount(0xffff_ffff_ffff_0000),
                        SlpToken::amount(0xffff_ffff_ffff_0001),
                        SlpToken::amount(2),
                    ],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Fungible,
                        token: SlpToken::amount(0xffff_ffff_ffff_0000),
                        group_token_id: None,
                    }),
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Fungible,
                        token: SlpToken::MINT_BATON,
                        group_token_id: None,
                    }),
                    None,
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Nft1Child,
                        token: SlpToken::amount(1),
                        group_token_id: Some(Box::new(TokenId::new(Sha256d::new([10; 32])))),
                    }),
                    None,
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Fungible,
                        token: SlpToken::amount(0xffff_ffff_ffff_0003),
                        group_token_id: None,
                    }),
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([3; 32])),
                        token_type: SlpTokenType::Nft1Group,
                        token: SlpToken::amount(100),
                        group_token_id: None,
                    })
                ],
            ),
            Err(SlpError::OutputSumExceedInputSum {
                input_sum: SlpAmount::new(0x1fffffffffffe0003),
                output_sum: SlpAmount::new(0x1fffffffffffe0004),
            }),
        );
        // Big NFT1 Group with off-by-one too little input tokens
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![
                        SlpToken::amount(1),
                        SlpToken::amount(0xffff_ffff_ffff_0000),
                        SlpToken::amount(0xffff_ffff_ffff_0001),
                        SlpToken::amount(2),
                    ],
                    slp_token_type: SlpTokenType::Nft1Group,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Nft1Group,
                        token: SlpToken::amount(0xffff_ffff_ffff_0000),
                        group_token_id: None,
                    }),
                    None,
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Nft1Child,
                        token: SlpToken::amount(1),
                        group_token_id: Some(Box::new(TokenId::new(Sha256d::new([10; 32])))),
                    }),
                    None,
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Nft1Group,
                        token: SlpToken::amount(0xffff_ffff_ffff_0003),
                        group_token_id: None,
                    }),
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([3; 32])),
                        token_type: SlpTokenType::Nft1Group,
                        token: SlpToken::amount(100),
                        group_token_id: None,
                    })
                ],
            ),
            Err(SlpError::OutputSumExceedInputSum {
                input_sum: SlpAmount::new(0x1fffffffffffe0003),
                output_sum: SlpAmount::new(0x1fffffffffffe0004),
            }),
        );
        Ok(())
    }

    #[test]
    fn test_validate_slp_tx_send_success() -> Result<()> {
        // Valid Fungible SEND with 0 inputs and outputs
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![SlpToken::EMPTY],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[None],
            ),
            Ok(SlpValidTxData {
                slp_tx_data: SlpTxData {
                    input_tokens: vec![SlpToken::EMPTY],
                    output_tokens: vec![SlpToken::EMPTY],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                    group_token_id: None,
                },
                slp_burns: vec![None],
            }),
        );
        // Valid NFT1 Group SEND with 0 inputs and outputs
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![SlpToken::EMPTY],
                    slp_token_type: SlpTokenType::Nft1Group,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[None],
            ),
            Ok(SlpValidTxData {
                slp_tx_data: SlpTxData {
                    input_tokens: vec![SlpToken::EMPTY],
                    output_tokens: vec![SlpToken::EMPTY],
                    slp_token_type: SlpTokenType::Nft1Group,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                    group_token_id: None,
                },
                slp_burns: vec![None],
            }),
        );
        // Valid NFT1 Child SEND with 0 inputs and outputs
        // This leaves group_token_id at None
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![SlpToken::EMPTY],
                    slp_token_type: SlpTokenType::Nft1Child,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[None],
            ),
            Ok(SlpValidTxData {
                slp_tx_data: SlpTxData {
                    input_tokens: vec![SlpToken::EMPTY],
                    output_tokens: vec![SlpToken::EMPTY],
                    slp_token_type: SlpTokenType::Nft1Child,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                    group_token_id: None,
                },
                slp_burns: vec![None],
            }),
        );
        // Fungible SEND sending 10 tokens and burning a MINT baton
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![SlpToken::amount(10)],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Fungible,
                        token: SlpToken::amount(10),
                        group_token_id: None,
                    }),
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Fungible,
                        token: SlpToken::MINT_BATON,
                        group_token_id: None,
                    })
                ],
            ),
            Ok(SlpValidTxData {
                slp_tx_data: SlpTxData {
                    input_tokens: vec![SlpToken::amount(10), SlpToken::EMPTY],
                    output_tokens: vec![SlpToken::amount(10)],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                    group_token_id: None,
                },
                slp_burns: vec![
                    None,
                    Some(Box::new(SlpBurn {
                        token: SlpToken::MINT_BATON,
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                    }))
                ],
            }),
        );
        // Big Fungible SEND with 64-bit overflow and partially burning tokens
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![
                        SlpToken::amount(0xffff_ffff_ffff_0000),
                        SlpToken::amount(0xffff_ffff_ffff_0002),
                        SlpToken::amount(1),
                    ],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Fungible,
                        token: SlpToken::amount(0xffff_ffff_ffff_0000),
                        group_token_id: None,
                    }),
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Fungible,
                        token: SlpToken::amount(0xefff_ffff_ffff_0000),
                        group_token_id: None,
                    }),
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Fungible,
                        token: SlpToken::amount(0x2fff_ffff_ffff_0000),
                        group_token_id: None,
                    }),
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Fungible,
                        token: SlpToken::amount(10),
                        group_token_id: None,
                    }),
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Nft1Child,
                        token: SlpToken::amount(10),
                        group_token_id: Some(Box::new(TokenId::new(Sha256d::new([10; 32])))),
                    }),
                ],
            ),
            Ok(SlpValidTxData {
                slp_tx_data: SlpTxData {
                    input_tokens: vec![
                        SlpToken::amount(0xffff_ffff_ffff_0000),
                        SlpToken::amount(0xefff_ffff_ffff_0000),
                        SlpToken::amount(0x2fff_ffff_ffff_0000),
                        SlpToken::amount(10),
                        SlpToken::EMPTY,
                    ],
                    output_tokens: vec![
                        SlpToken::amount(0xffff_ffff_ffff_0000),
                        SlpToken::amount(0xffff_ffff_ffff_0002),
                        SlpToken::amount(1),
                    ],
                    slp_token_type: SlpTokenType::Fungible,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                    group_token_id: None,
                },
                slp_burns: vec![
                    None,
                    None,
                    Some(Box::new(SlpBurn {
                        token: SlpToken::amount(0x1fff_ffff_fffe_fffd),
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                    })),
                    Some(Box::new(SlpBurn {
                        token: SlpToken::amount(10),
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                    })),
                    Some(Box::new(SlpBurn {
                        token: SlpToken::amount(10),
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                    })),
                ],
            }),
        );
        // Big NFT1 Group SEND with 64-bit overflow and partially burning tokens
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![
                        SlpToken::amount(0xffff_ffff_ffff_0000),
                        SlpToken::amount(0xffff_ffff_ffff_0002),
                        SlpToken::amount(1),
                    ],
                    slp_token_type: SlpTokenType::Nft1Group,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Nft1Group,
                        token: SlpToken::amount(0xffff_ffff_ffff_0000),
                        group_token_id: None,
                    }),
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Nft1Group,
                        token: SlpToken::amount(0xefff_ffff_ffff_0000),
                        group_token_id: None,
                    }),
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Nft1Group,
                        token: SlpToken::amount(0x2fff_ffff_ffff_0000),
                        group_token_id: None,
                    }),
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Nft1Group,
                        token: SlpToken::amount(10),
                        group_token_id: None,
                    }),
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Nft1Child,
                        token: SlpToken::amount(10),
                        group_token_id: Some(Box::new(TokenId::new(Sha256d::new([10; 32])))),
                    }),
                ],
            ),
            Ok(SlpValidTxData {
                slp_tx_data: SlpTxData {
                    input_tokens: vec![
                        SlpToken::amount(0xffff_ffff_ffff_0000),
                        SlpToken::amount(0xefff_ffff_ffff_0000),
                        SlpToken::amount(0x2fff_ffff_ffff_0000),
                        SlpToken::amount(10),
                        SlpToken::EMPTY,
                    ],
                    output_tokens: vec![
                        SlpToken::amount(0xffff_ffff_ffff_0000),
                        SlpToken::amount(0xffff_ffff_ffff_0002),
                        SlpToken::amount(1),
                    ],
                    slp_token_type: SlpTokenType::Nft1Group,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                    group_token_id: None,
                },
                slp_burns: vec![
                    None,
                    None,
                    Some(Box::new(SlpBurn {
                        token: SlpToken::amount(0x1fff_ffff_fffe_fffd),
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                    })),
                    Some(Box::new(SlpBurn {
                        token: SlpToken::amount(10),
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                    })),
                    Some(Box::new(SlpBurn {
                        token: SlpToken::amount(10),
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                    })),
                ],
            }),
        );
        // Big NFT1 Child SEND with two 0 value NFT1 Child inputs
        assert_eq!(
            validate_slp_tx(
                SlpParseData {
                    output_tokens: vec![SlpToken::EMPTY, SlpToken::amount(1), SlpToken::EMPTY],
                    slp_token_type: SlpTokenType::Nft1Child,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                },
                &[
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Nft1Child,
                        token: SlpToken::EMPTY,
                        group_token_id: Some(Box::new(TokenId::new(Sha256d::new([10; 32])))),
                    }),
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Nft1Group,
                        token: SlpToken::amount(0xefff_ffff_ffff_0000),
                        group_token_id: None,
                    }),
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Nft1Child,
                        token: SlpToken::amount(1),
                        group_token_id: Some(Box::new(TokenId::new(Sha256d::new([10; 32])))),
                    }),
                    Some(SlpSpentOutput {
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                        token_type: SlpTokenType::Nft1Child,
                        token: SlpToken::EMPTY,
                        group_token_id: None,
                    }),
                ],
            ),
            Ok(SlpValidTxData {
                slp_tx_data: SlpTxData {
                    input_tokens: vec![
                        SlpToken::EMPTY,
                        SlpToken::EMPTY,
                        SlpToken::amount(1),
                        SlpToken::EMPTY,
                    ],
                    output_tokens: vec![SlpToken::EMPTY, SlpToken::amount(1), SlpToken::EMPTY],
                    slp_token_type: SlpTokenType::Nft1Child,
                    slp_tx_type: SlpTxType::Send,
                    token_id: TokenId::new(Sha256d::new([4; 32])),
                    group_token_id: Some(Box::new(TokenId::new(Sha256d::new([10; 32])))),
                },
                slp_burns: vec![
                    None,
                    Some(Box::new(SlpBurn {
                        token: SlpToken::amount(0xefff_ffff_ffff_0000),
                        token_id: TokenId::new(Sha256d::new([4; 32])),
                    })),
                    None,
                    None,
                ],
            }),
        );
        Ok(())
    }
}
