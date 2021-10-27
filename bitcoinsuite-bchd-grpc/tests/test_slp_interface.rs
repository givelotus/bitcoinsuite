use std::{collections::HashSet, time::Duration};

use bitcoinsuite_bchd_grpc::BchdSlpInterface;
use bitcoinsuite_core::{
    ecc::Ecc, AddressType, BitcoinCode, Bytes, CashAddress, Hashed, Net, Network, OutPoint,
    P2PKHSignatory, Script, SequenceNo, Sha256d, ShaRmd160, SigHashType, SignData, SignField,
    TxBuilder, TxInput, TxOutput, UnhashedTx, Utxo, ECASH,
};
use bitcoinsuite_ecc_secp256k1::EccSecp256k1;
use bitcoinsuite_error::Result;
use bitcoinsuite_slp::{
    genesis_opreturn, mint_opreturn, send_opreturn, SlpAmount, SlpBurn, SlpGenesisInfo,
    SlpNodeInterface, SlpToken, SlpTokenType, SlpTx, SlpTxData, SlpTxType, SlpUtxo, TokenId,
};
use bitcoinsuite_test_utils_blockchain::{build_tx, setup_xec_chain};
use futures::StreamExt;
use pretty_assertions::assert_eq;
use tokio::time::timeout;

#[allow(clippy::mutable_key_type)]
async fn test_slp_interface() -> Result<()> {
    let ecc = EccSecp256k1::default();
    let redeem_script = Script::from_static_slice(&[0x51]);
    let redeem_script2 = Script::from_static_slice(&[0x52]);
    let (bitcoind, mut bchd, mut utxos) = setup_xec_chain(10, &redeem_script).await?;

    let node = BchdSlpInterface::new(bchd.client().clone(), Net::Regtest);
    let seckey = ecc.seckey_from_array([3; 32])?;
    let pubkey = ecc.derive_pubkey(&seckey);
    let pkh = ShaRmd160::digest(pubkey.as_slice().into());
    let address = CashAddress::from_hash(ECASH, AddressType::P2PKH, pkh);
    let tx_handle = tokio::spawn({
        let node = node.clone();
        let address = address.clone();
        async move { node.address_tx_stream(&address).await }
    });
    // Allow other thread to listen to stream before sending any txs
    tokio::time::sleep(Duration::from_secs(1)).await;

    let (outpoint, miner_value) = utxos.pop().unwrap();
    let utxo_value = miner_value / 2 - 10_000;
    let tx = build_tx(
        outpoint,
        &redeem_script,
        vec![
            TxOutput {
                value: utxo_value,
                script: address.to_script(),
            },
            TxOutput {
                value: utxo_value,
                script: address.to_script(),
            },
            TxOutput {
                value: 0,
                script: Script::opreturn(&[&[0; 100]]),
            },
        ],
    );
    // submit on both BCHD and bitcoind:
    // - BCHD checks for valid SLP
    // - bitcoind checks for undersize etc. and other rules
    let txid = node.submit_tx(tx.ser().to_vec()).await?;
    bitcoind.cmd_string("sendrawtransaction", &[&tx.ser().hex()])?;
    let mut tx_stream = tx_handle.await??;
    let actual_tx = timeout(Duration::from_secs(3), tx_stream.next())
        .await?
        .expect("Stream ended unexpectedly")?;
    let expected_tx = SlpTx::new(tx, None, vec![None]);
    assert_eq!(expected_tx, actual_tx);
    let address_utxos = node.address_utxos(&address).await?;
    let sats_utxo1 = SlpUtxo {
        utxo: Utxo {
            outpoint: OutPoint {
                txid: txid.clone(),
                out_idx: 0,
            },
            script: address.to_script(),
            value: utxo_value,
        },
        token: SlpToken::EMPTY,
        token_id: None,
    };
    let sats_utxo2 = SlpUtxo {
        utxo: Utxo {
            outpoint: OutPoint { txid, out_idx: 1 },
            script: address.to_script(),
            value: utxo_value,
        },
        token: SlpToken::EMPTY,
        token_id: None,
    };
    assert_eq!(address_utxos, vec![sats_utxo1.clone(), sats_utxo2.clone()]);

    {
        let (utxo_outpoint, utxo_value) = utxos.pop().unwrap();
        let genesis_info = SlpGenesisInfo {
            token_ticker: Bytes::from_bytes(b"TEST".as_ref()),
            token_name: Bytes::from_bytes(b"Test".as_ref()),
            token_document_url: Bytes::from_bytes(b"example.com".as_ref()),
            token_document_hash: None,
            decimals: 0,
        };
        let genesis_output_value = utxo_value / 2 - 10_000;
        let genesis_tx = build_tx(
            utxo_outpoint,
            &redeem_script,
            vec![
                TxOutput {
                    value: 0,
                    script: genesis_opreturn(&genesis_info, SlpTokenType::Fungible, Some(2), 20),
                },
                TxOutput {
                    value: genesis_output_value,
                    script: address.to_script(),
                },
                TxOutput {
                    value: genesis_output_value,
                    script: redeem_script2.to_p2sh(),
                },
            ],
        );
        let genesis_txid = Sha256d::digest(genesis_tx.ser());
        let token_id = TokenId::new(genesis_txid.clone());

        node.submit_tx(genesis_tx.ser().to_vec()).await?;
        bitcoind.cmd_string("sendrawtransaction", &[&genesis_tx.ser().hex()])?;
        let actual_tx = timeout(Duration::from_secs(3), tx_stream.next())
            .await?
            .expect("Stream ended unexpectedly")?;
        let expected_tx = SlpTx::new(
            genesis_tx,
            Some(SlpTxData {
                input_tokens: vec![SlpToken::EMPTY],
                output_tokens: vec![SlpToken::EMPTY, SlpToken::amount(20), SlpToken::MINT_BATON],
                slp_token_type: SlpTokenType::Fungible,
                slp_tx_type: SlpTxType::Genesis(Box::new(genesis_info)),
                token_id: token_id.clone(),
                group_token_id: None,
            }),
            vec![None],
        );
        assert_eq!(expected_tx, actual_tx);
        let address_utxos = node
            .address_utxos(&address)
            .await?
            .into_iter()
            .collect::<HashSet<_>>();
        assert_eq!(
            address_utxos,
            [
                sats_utxo1,
                sats_utxo2.clone(),
                SlpUtxo {
                    utxo: Utxo {
                        outpoint: OutPoint {
                            txid: genesis_txid.clone(),
                            out_idx: 1
                        },
                        script: address.to_script(),
                        value: genesis_output_value
                    },
                    token: SlpToken::amount(20),
                    token_id: Some(token_id.clone()),
                },
            ]
            .into_iter()
            .collect(),
        );
        let mint_sh = ShaRmd160::digest(redeem_script2.bytecode().clone());
        let mint_address = CashAddress::from_hash(ECASH, AddressType::P2SH, mint_sh);
        let genesis_mint_utxos = node.address_utxos(&mint_address).await?;
        assert_eq!(
            genesis_mint_utxos,
            vec![SlpUtxo {
                utxo: Utxo {
                    outpoint: OutPoint {
                        txid: genesis_txid.clone(),
                        out_idx: 2
                    },
                    script: mint_address.to_script(),
                    value: genesis_output_value
                },
                token: SlpToken::MINT_BATON,
                token_id: Some(token_id.clone()),
            }]
        );

        let mint_output_value = genesis_output_value / 2 - 10_000;
        let mint_tx = build_tx(
            OutPoint {
                txid: genesis_txid,
                out_idx: 2,
            },
            &redeem_script2,
            vec![
                TxOutput {
                    value: 0,
                    script: mint_opreturn(&token_id, SlpTokenType::Fungible, Some(2), 10),
                },
                TxOutput {
                    value: mint_output_value,
                    script: address.to_script(),
                },
                TxOutput {
                    value: mint_output_value,
                    script: redeem_script.to_p2sh(),
                },
            ],
        );
        let mint_txid = Sha256d::digest(mint_tx.ser());

        node.submit_tx(mint_tx.ser().to_vec()).await?;
        bitcoind.cmd_string("sendrawtransaction", &[&mint_tx.ser().hex()])?;
        let actual_tx = timeout(Duration::from_secs(3), tx_stream.next())
            .await?
            .expect("Stream ended unexpectedly")?;
        let expected_tx = SlpTx::new(
            mint_tx,
            Some(SlpTxData {
                input_tokens: vec![SlpToken::MINT_BATON],
                output_tokens: vec![SlpToken::EMPTY, SlpToken::amount(10), SlpToken::MINT_BATON],
                slp_token_type: SlpTokenType::Fungible,
                slp_tx_type: SlpTxType::Mint,
                token_id: token_id.clone(),
                group_token_id: None,
            }),
            vec![None],
        );
        assert_eq!(expected_tx, actual_tx);

        let send_output_value = mint_output_value / 2 - 10_000;
        let send_tx = UnhashedTx {
            version: 1,
            inputs: vec![TxInput {
                prev_out: OutPoint {
                    txid: mint_txid,
                    out_idx: 1,
                },
                script: Script::default(),
                sequence: SequenceNo::finalized(),
                sign_data: Some(SignData::new(vec![
                    SignField::Value(mint_output_value),
                    SignField::OutputScript(address.to_script()),
                ])),
            }],
            outputs: vec![
                TxOutput {
                    value: 0,
                    script: send_opreturn(
                        &token_id,
                        SlpTokenType::Fungible,
                        &[SlpAmount::new(7), SlpAmount::new(3)],
                    ),
                },
                TxOutput {
                    value: send_output_value,
                    script: redeem_script.to_p2sh(),
                },
                TxOutput {
                    value: send_output_value,
                    script: redeem_script.to_p2sh(),
                },
            ],
            lock_time: 0,
        };
        let mut tx_builder = TxBuilder::from_tx(send_tx);
        *tx_builder.inputs[0].signatory_mut() = Some(Box::new(P2PKHSignatory {
            seckey: seckey.clone(),
            pubkey,
            sig_hash_type: SigHashType::ALL_BIP143,
        }));
        let mut send_tx = tx_builder.sign(&ecc, 1000, Network::XEC.dust_amount())?;
        send_tx.inputs[0].sign_data = None;
        let send_txid = Sha256d::digest(send_tx.ser());

        node.submit_tx(send_tx.ser().to_vec()).await?;
        bitcoind.cmd_string("sendrawtransaction", &[&send_tx.ser().hex()])?;
        let actual_tx = timeout(Duration::from_secs(3), tx_stream.next())
            .await?
            .expect("Stream ended unexpectedly")?;
        let expected_tx = SlpTx::new(
            send_tx,
            Some(SlpTxData {
                input_tokens: vec![SlpToken::amount(10)],
                output_tokens: vec![SlpToken::EMPTY, SlpToken::amount(7), SlpToken::amount(3)],
                slp_token_type: SlpTokenType::Fungible,
                slp_tx_type: SlpTxType::Send,
                token_id: token_id.clone(),
                group_token_id: None,
            }),
            vec![None],
        );
        assert_eq!(expected_tx, actual_tx);

        let burn_output_value = send_output_value - 10_000;
        let mut genesis_txid_reverse = token_id.hash().byte_array().array();
        genesis_txid_reverse.reverse();
        let token_id_reverse = TokenId::new(Sha256d::new(genesis_txid_reverse));
        let burn_tx = build_tx(
            OutPoint {
                txid: send_txid,
                out_idx: 1,
            },
            &redeem_script,
            vec![
                TxOutput {
                    value: 0,
                    script: send_opreturn(
                        &token_id_reverse,
                        SlpTokenType::Fungible,
                        &[SlpAmount::new(7)],
                    ),
                },
                TxOutput {
                    value: burn_output_value,
                    script: address.to_script(),
                },
            ],
        );
        // submit only through bitcoind, which doesn't check for SLP burns
        bitcoind.cmd_string("sendrawtransaction", &[&burn_tx.ser().hex()])?;
        let actual_tx = timeout(Duration::from_secs(20), tx_stream.next())
            .await?
            .expect("Stream ended unexpectedly")?;
        let expected_tx = SlpTx::new(
            burn_tx,
            Some(SlpTxData {
                input_tokens: vec![SlpToken::EMPTY],
                output_tokens: vec![SlpToken::EMPTY, SlpToken::EMPTY],
                slp_token_type: SlpTokenType::Fungible,
                slp_tx_type: SlpTxType::Send,
                token_id: token_id_reverse.clone(),
                group_token_id: None,
            }),
            vec![Some(Box::new(SlpBurn {
                token: SlpToken::amount(7),
                token_id: token_id.clone(),
            }))],
        );
        assert_eq!(expected_tx, actual_tx);
    }

    {
        let (utxo_outpoint, utxo_value) = utxos.pop().unwrap();
        let genesis_info = SlpGenesisInfo {
            token_ticker: Bytes::from_bytes(b"NFTTEST".as_ref()),
            token_name: Bytes::from_bytes(b"NftTest".as_ref()),
            token_document_url: Bytes::from_bytes(b"nft.example.com".as_ref()),
            token_document_hash: None,
            decimals: 0,
        };
        let genesis_output_value = utxo_value / 2 - 10_000;
        let genesis_tx = build_tx(
            utxo_outpoint,
            &redeem_script,
            vec![
                TxOutput {
                    value: 0,
                    script: genesis_opreturn(&genesis_info, SlpTokenType::Nft1Group, Some(2), 20),
                },
                TxOutput {
                    value: genesis_output_value,
                    script: address.to_script(),
                },
                TxOutput {
                    value: genesis_output_value,
                    script: redeem_script2.to_p2sh(),
                },
            ],
        );
        let genesis_txid = Sha256d::digest(genesis_tx.ser());
        let token_id = TokenId::new(genesis_txid.clone());

        node.submit_tx(genesis_tx.ser().to_vec()).await?;
        bitcoind.cmd_string("sendrawtransaction", &[&genesis_tx.ser().hex()])?;
        let actual_tx = timeout(Duration::from_secs(3), tx_stream.next())
            .await?
            .expect("Stream ended unexpectedly")?;
        let expected_tx = SlpTx::new(
            genesis_tx,
            Some(SlpTxData {
                input_tokens: vec![SlpToken::EMPTY],
                output_tokens: vec![SlpToken::EMPTY, SlpToken::amount(20), SlpToken::MINT_BATON],
                slp_token_type: SlpTokenType::Nft1Group,
                slp_tx_type: SlpTxType::Genesis(Box::new(genesis_info)),
                token_id: token_id.clone(),
                group_token_id: None,
            }),
            vec![None],
        );
        assert_eq!(expected_tx, actual_tx);
        let mint_sh = ShaRmd160::digest(redeem_script2.bytecode().clone());
        let mint_address = CashAddress::from_hash(ECASH, AddressType::P2SH, mint_sh);
        let genesis_mint_utxos = node.address_utxos(&mint_address).await?;
        assert_eq!(
            genesis_mint_utxos,
            vec![SlpUtxo {
                utxo: Utxo {
                    outpoint: OutPoint {
                        txid: genesis_txid.clone(),
                        out_idx: 2
                    },
                    script: mint_address.to_script(),
                    value: genesis_output_value
                },
                token: SlpToken::MINT_BATON,
                token_id: Some(token_id.clone()),
            }]
        );

        let mint_output_value = genesis_output_value / 2 - 10_000;
        let mint_tx = build_tx(
            OutPoint {
                txid: genesis_txid,
                out_idx: 2,
            },
            &redeem_script2,
            vec![
                TxOutput {
                    value: 0,
                    script: mint_opreturn(&token_id, SlpTokenType::Nft1Group, Some(2), 10),
                },
                TxOutput {
                    value: mint_output_value,
                    script: address.to_script(),
                },
                TxOutput {
                    value: mint_output_value,
                    script: redeem_script.to_p2sh(),
                },
            ],
        );
        let mint_txid = Sha256d::digest(mint_tx.ser());

        node.submit_tx(mint_tx.ser().to_vec()).await?;
        bitcoind.cmd_string("sendrawtransaction", &[&mint_tx.ser().hex()])?;
        let actual_tx = timeout(Duration::from_secs(3), tx_stream.next())
            .await?
            .expect("Stream ended unexpectedly")?;
        let expected_tx = SlpTx::new(
            mint_tx,
            Some(SlpTxData {
                input_tokens: vec![SlpToken::MINT_BATON],
                output_tokens: vec![SlpToken::EMPTY, SlpToken::amount(10), SlpToken::MINT_BATON],
                slp_token_type: SlpTokenType::Nft1Group,
                slp_tx_type: SlpTxType::Mint,
                token_id: token_id.clone(),
                group_token_id: None,
            }),
            vec![None],
        );
        assert_eq!(expected_tx, actual_tx);

        let send_output_value = mint_output_value / 2 - 10_000;
        let send_tx = UnhashedTx {
            version: 1,
            inputs: vec![TxInput {
                prev_out: OutPoint {
                    txid: mint_txid,
                    out_idx: 1,
                },
                script: Script::default(),
                sequence: SequenceNo::finalized(),
                sign_data: Some(SignData::new(vec![
                    SignField::Value(mint_output_value),
                    SignField::OutputScript(address.to_script()),
                ])),
            }],
            outputs: vec![
                TxOutput {
                    value: 0,
                    script: send_opreturn(
                        &token_id,
                        SlpTokenType::Nft1Group,
                        &[SlpAmount::new(1), SlpAmount::new(9)],
                    ),
                },
                TxOutput {
                    value: send_output_value,
                    script: redeem_script.to_p2sh(),
                },
                TxOutput {
                    value: send_output_value,
                    script: redeem_script.to_p2sh(),
                },
            ],
            lock_time: 0,
        };
        let mut tx_builder = TxBuilder::from_tx(send_tx);
        *tx_builder.inputs[0].signatory_mut() = Some(Box::new(P2PKHSignatory {
            seckey: seckey.clone(),
            pubkey,
            sig_hash_type: SigHashType::ALL_BIP143,
        }));
        let mut send_tx = tx_builder.sign(&ecc, 1000, Network::XEC.dust_amount())?;
        send_tx.inputs[0].sign_data = None;
        let send_txid = Sha256d::digest(send_tx.ser());

        node.submit_tx(send_tx.ser().to_vec()).await?;
        bitcoind.cmd_string("sendrawtransaction", &[&send_tx.ser().hex()])?;
        let actual_tx = timeout(Duration::from_secs(3), tx_stream.next())
            .await?
            .expect("Stream ended unexpectedly")?;
        let expected_tx = SlpTx::new(
            send_tx,
            Some(SlpTxData {
                input_tokens: vec![SlpToken::amount(10)],
                output_tokens: vec![SlpToken::EMPTY, SlpToken::amount(1), SlpToken::amount(9)],
                slp_token_type: SlpTokenType::Nft1Group,
                slp_tx_type: SlpTxType::Send,
                token_id: token_id.clone(),
                group_token_id: None,
            }),
            vec![None],
        );
        assert_eq!(expected_tx, actual_tx);

        let child_genesis_info = SlpGenesisInfo {
            token_ticker: Bytes::from_bytes(b"NFTCHILD".as_ref()),
            token_name: Bytes::from_bytes(b"NftChild".as_ref()),
            token_document_url: Bytes::from_bytes(b"nft.child.example.com".as_ref()),
            token_document_hash: None,
            decimals: 0,
        };
        let genesis_output_value = send_output_value - 10_000;
        let genesis_tx = build_tx(
            OutPoint {
                txid: send_txid,
                out_idx: 1,
            },
            &redeem_script,
            vec![
                TxOutput {
                    value: 0,
                    script: genesis_opreturn(&child_genesis_info, SlpTokenType::Nft1Child, None, 1),
                },
                TxOutput {
                    value: genesis_output_value,
                    script: address.to_script(),
                },
            ],
        );
        let child_genesis_txid = Sha256d::digest(genesis_tx.ser());
        let child_token_id = TokenId::new(child_genesis_txid.clone());

        node.submit_tx(genesis_tx.ser().to_vec()).await?;
        bitcoind.cmd_string("sendrawtransaction", &[&genesis_tx.ser().hex()])?;
        let actual_tx = timeout(Duration::from_secs(3), tx_stream.next())
            .await?
            .expect("Stream ended unexpectedly")?;
        let expected_tx = SlpTx::new(
            genesis_tx,
            Some(SlpTxData {
                input_tokens: vec![SlpToken::amount(1)],
                output_tokens: vec![SlpToken::EMPTY, SlpToken::amount(1)],
                slp_token_type: SlpTokenType::Nft1Child,
                slp_tx_type: SlpTxType::Genesis(Box::new(child_genesis_info)),
                token_id: child_token_id.clone(),
                group_token_id: Some(Box::new(token_id.clone())),
            }),
            vec![None],
        );
        assert_eq!(expected_tx, actual_tx);

        let send_output_value = genesis_output_value - 10_000;
        let child_send_tx = UnhashedTx {
            version: 1,
            inputs: vec![TxInput {
                prev_out: OutPoint {
                    txid: child_genesis_txid,
                    out_idx: 1,
                },
                script: Script::default(),
                sequence: SequenceNo::finalized(),
                sign_data: Some(SignData::new(vec![
                    SignField::Value(genesis_output_value),
                    SignField::OutputScript(address.to_script()),
                ])),
            }],
            outputs: vec![
                TxOutput {
                    value: 0,
                    script: send_opreturn(
                        &child_token_id,
                        SlpTokenType::Nft1Child,
                        &[SlpAmount::new(1)],
                    ),
                },
                TxOutput {
                    value: send_output_value,
                    script: address.to_script(),
                },
            ],
            lock_time: 0,
        };
        let mut tx_builder = TxBuilder::from_tx(child_send_tx);
        *tx_builder.inputs[0].signatory_mut() = Some(Box::new(P2PKHSignatory {
            seckey: seckey.clone(),
            pubkey,
            sig_hash_type: SigHashType::ALL_BIP143,
        }));
        let mut child_send_tx = tx_builder.sign(&ecc, 1000, Network::XEC.dust_amount())?;
        child_send_tx.inputs[0].sign_data = None;

        node.submit_tx(child_send_tx.ser().to_vec()).await?;
        bitcoind.cmd_string("sendrawtransaction", &[&child_send_tx.ser().hex()])?;
        let actual_tx = timeout(Duration::from_secs(3), tx_stream.next())
            .await?
            .expect("Stream ended unexpectedly")?;
        let expected_tx = SlpTx::new(
            child_send_tx,
            Some(SlpTxData {
                input_tokens: vec![SlpToken::amount(1)],
                output_tokens: vec![SlpToken::EMPTY, SlpToken::amount(1)],
                slp_token_type: SlpTokenType::Nft1Child,
                slp_tx_type: SlpTxType::Send,
                token_id: child_token_id.clone(),
                group_token_id: Some(Box::new(token_id.clone())),
            }),
            vec![None],
        );
        assert_eq!(expected_tx, actual_tx);
    }

    Ok(())
}

#[test]
fn run_slp_interface_tests() -> Result<()> {
    bitcoinsuite_error::install()?;
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(test_slp_interface())?;
    Ok(())
}
