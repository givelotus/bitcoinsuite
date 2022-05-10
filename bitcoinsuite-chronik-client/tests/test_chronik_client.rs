use bitcoinsuite_chronik_client::{proto, ChronikClient, ChronikClientError, ScriptType};
use bitcoinsuite_core::{Hashed, Sha256d};
use bitcoinsuite_error::Result;

use pretty_assertions::assert_eq;
use reqwest::StatusCode;

const CHRONIK_URL: &str = "https://chronik.be.cash/xpi";
const GENESIS_PK_HEX: &str = "04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb6\
                              49f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
pub async fn test_broadcast_tx() -> Result<()> {
    let client = ChronikClient::new(CHRONIK_URL.to_string())?;
    let response = client
        .broadcast_tx(hex::decode("00000000")?)
        .await
        .unwrap_err()
        .downcast::<ChronikClientError>()?;
    let error_msg =
        "Invalid tx encoding: Bytes error: Index 1 is out of bounds for array with length 0";
    assert_eq!(
        response,
        ChronikClientError::ChronikError {
            status_code: StatusCode::BAD_REQUEST,
            error: proto::Error {
                error_code: "invalid-tx-encoding".to_string(),
                msg: error_msg.to_string(),
                is_user_error: true,
            },
            error_msg: error_msg.to_string(),
        },
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
pub async fn test_broadcast_txs() -> Result<()> {
    let client = ChronikClient::new(CHRONIK_URL.to_string())?;
    let response = client
        .broadcast_txs(vec![hex::decode("00000000")?])
        .await
        .unwrap_err()
        .downcast::<ChronikClientError>()?;
    let error_msg =
        "Invalid tx encoding: Bytes error: Index 1 is out of bounds for array with length 0";
    assert_eq!(
        response,
        ChronikClientError::ChronikError {
            status_code: StatusCode::BAD_REQUEST,
            error: proto::Error {
                error_code: "invalid-tx-encoding".to_string(),
                msg: error_msg.to_string(),
                is_user_error: true,
            },
            error_msg: error_msg.to_string(),
        },
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
pub async fn test_blockchain_info() -> Result<()> {
    let client = ChronikClient::new(CHRONIK_URL.to_string())?;
    let blockchain_info = client.blockchain_info().await?;
    assert!(blockchain_info.tip_height > 243892);
    assert_eq!(blockchain_info.tip_hash[28..], [0; 4]);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
pub async fn test_block() -> Result<()> {
    let client = ChronikClient::new(CHRONIK_URL.to_string())?;
    let block_hash =
        Sha256d::from_hex_be("0000000000124d5456d0c9946ad1bac56d481aa3657b4a160157b5102038e380")?;
    let prev_block_hash =
        Sha256d::from_hex_be("0000000000125aa2c3d6dce7c1f204aa588790efed73086c28938b93cb985c0a")?;
    let expected_height = 129_113;
    let block = client.block_by_hash(&block_hash).await?;
    assert_eq!(
        block.block_info,
        Some(proto::BlockInfo {
            hash: block_hash.as_slice().to_vec(),
            prev_hash: prev_block_hash.as_slice().to_vec(),
            height: expected_height,
            n_bits: 0x1b14_5080,
            timestamp: 1_638_416_969,
            block_size: 5067,
            num_txs: 2,
            num_inputs: 30,
            num_outputs: 17,
            sum_input_sats: 32_524_544_607,
            sum_coinbase_output_sats: 2_250_449_185,
            sum_normal_output_sats: 32_524_540_237,
            sum_burned_sats: 0,
        }),
    );
    assert_eq!(
        block.block_details,
        Some(proto::BlockDetails {
            version: 1,
            merkle_root: Sha256d::from_hex_be(
                "c918c874d0231b6cdbac222ec7bde52cfaa6c5eafdc1a0c2bc5ed2d2ffdb40d7"
            )?
            .as_slice()
            .to_vec(),
            nonce: 5831068090634202948,
            median_timestamp: 1638416423,
        }),
    );
    assert_eq!(client.block_by_height(expected_height).await?, block,);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
pub async fn test_blocks() -> Result<()> {
    let client = ChronikClient::new(CHRONIK_URL.to_string())?;
    let blocks = client.blocks(129_113, 129_120).await?;
    assert_eq!(blocks.len(), 8);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
pub async fn test_tx_missing() -> Result<()> {
    let client = ChronikClient::new(CHRONIK_URL.to_string())?;
    let err = client
        .tx(&Sha256d::new([0; 32]))
        .await
        .unwrap_err()
        .downcast::<ChronikClientError>()?;
    let error_msg =
        "Txid not found: 0000000000000000000000000000000000000000000000000000000000000000";
    assert_eq!(
        err,
        ChronikClientError::ChronikError {
            status_code: StatusCode::NOT_FOUND,
            error_msg: error_msg.to_string(),
            error: proto::Error {
                error_code: "tx-not-found".to_string(),
                msg: error_msg.to_string(),
                is_user_error: true,
            },
        },
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
pub async fn test_tx() -> Result<()> {
    let client = ChronikClient::new(CHRONIK_URL.to_string())?;
    let block_hash =
        Sha256d::from_hex_be("000000000784d2162c9759f2b833a9dcffffc7700b4a6419edbc3ac8b3d604a3")?;
    let txid =
        Sha256d::from_hex_be("11a47493b3eac2762ad0e937ed8a2c4183f48248e5a78cffd0fadd58d21ac659")?;
    let actual_tx = client.tx(&txid).await?;
    let expected_tx = proto::Tx {
        txid: txid.as_slice().to_vec(),
        version: 2,
        inputs: vec![proto::TxInput {
            prev_out: Some(proto::OutPoint {
                txid: Sha256d::from_hex_be(
                    "cf0a7c88e4e401cb0929c07c642c627df00e0305aa99fea3f5e2ba6c1e307874",
                )?
                .as_slice()
                .to_vec(),
                out_idx: 1,
            }),
            input_script: hex::decode(
                "47304402204c79304e40bd49748e88c5f42d6d731857cd34c90e8de4e55dafe4215c7941\
                 e5022029b8ea0578f64f195101a38067f96904b42e7df2d17dad98922cf0c9149645f041\
                 21029bd3f4a509a586da48750b4164463f602bbe4e278ed602825f63f25f73cfc363",
            )?,
            output_script: hex::decode("76a9146c311bcd8389619a7e97c362fa04720e172b228f88ac")?,
            value: 130201868,
            sequence_no: 0xfffffffe,
            slp_burn: None,
            slp_token: None,
        }],
        outputs: vec![
            proto::TxOutput {
                value: 30_201_643,
                output_script: hex::decode("76a914c9efa1f37f0105c1cdb5e95672dc3d3d9405f04388ac")?,
                slp_token: None,
                spent_by: Some(proto::OutPoint {
                    txid: Sha256d::from_hex_be(
                        "53d4d20f224fe3c98b72be664aa7a9254a3f804318a3878190179f208077c020",
                    )?
                    .as_slice()
                    .to_vec(),
                    out_idx: 87,
                }),
            },
            proto::TxOutput {
                value: 100_000_000,
                output_script: hex::decode("76a9141e81a0e8c5e4f8c7dd40db10b84e3a3b4c568c0788ac")?,
                slp_token: None,
                spent_by: Some(proto::OutPoint {
                    txid: Sha256d::from_hex_be(
                        "66e6d0fa4fab71aeb7002ef8ae7f11f8ee166e9ec8d53188fcff7581aa7e6ed2",
                    )?
                    .as_slice()
                    .to_vec(),
                    out_idx: 96,
                }),
            },
        ],
        lock_time: 111,
        slp_tx_data: None,
        slp_error_msg: "".to_string(),
        block: Some(proto::BlockMetadata {
            height: 112,
            hash: block_hash.as_slice().to_vec(),
            timestamp: 1_624_249_618,
        }),
        time_first_seen: 0,
        network: proto::Network::Xpi as i32,
    };
    assert_eq!(actual_tx, expected_tx);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
pub async fn test_token() -> Result<()> {
    let client = ChronikClient::new("https://chronik.be.cash/xec".to_string())?;
    let token_id =
        Sha256d::from_hex_be("0daf200e3418f2df1158efef36fbb507f12928f1fdcf3543703e64e75a4a9073")?;
    let token = client.token(&token_id).await?;
    assert_eq!(
        token.slp_tx_data,
        Some(proto::SlpTxData {
            slp_meta: Some(proto::SlpMeta {
                token_type: proto::SlpTokenType::Fungible as i32,
                tx_type: proto::SlpTxType::Genesis as i32,
                token_id: token_id.to_vec_be(),
                group_token_id: vec![],
            }),
            genesis_info: Some(proto::SlpGenesisInfo {
                token_ticker: b"USDR".to_vec(),
                token_name: b"RaiUSD".to_vec(),
                token_document_url: b"https://www.raiusd.co/etoken".to_vec(),
                token_document_hash: vec![],
                decimals: 4,
            }),
        }),
    );
    let token_stats = token.token_stats.unwrap();
    assert!(!token_stats.total_minted.is_empty());
    assert!(!token_stats.total_burned.is_empty());
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
pub async fn test_validate_utxos() -> Result<()> {
    let client = ChronikClient::new(CHRONIK_URL.to_string())?;
    let utxo_states = client
        .validate_utxos(vec![
            proto::OutPoint {
                txid: Sha256d::from_hex_be(
                    "7455e298a18829d294441acaabcb854e04b2dd609e29c6cc805392271f9c53ea",
                )?
                .as_slice()
                .to_vec(),
                out_idx: 1,
            },
            proto::OutPoint {
                txid: Sha256d::from_hex_be(
                    "11a47493b3eac2762ad0e937ed8a2c4183f48248e5a78cffd0fadd58d21ac659",
                )?
                .as_slice()
                .to_vec(),
                out_idx: 1,
            },
            proto::OutPoint {
                txid: Sha256d::from_hex_be(
                    "11a47493b3eac2762ad0e937ed8a2c4183f48248e5a78cffd0fadd58d21ac659",
                )?
                .as_slice()
                .to_vec(),
                out_idx: 100,
            },
            proto::OutPoint {
                txid: Sha256d::from_hex_be(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                )?
                .as_slice()
                .to_vec(),
                out_idx: 0,
            },
        ])
        .await?;
    assert_eq!(
        utxo_states,
        vec![
            proto::UtxoState {
                height: 0, // GENESIS
                is_confirmed: true,
                state: proto::UtxoStateVariant::Unspent as i32,
            },
            proto::UtxoState {
                height: 112,
                is_confirmed: true,
                state: proto::UtxoStateVariant::Spent as i32,
            },
            proto::UtxoState {
                height: 112,
                is_confirmed: true,
                state: proto::UtxoStateVariant::NoSuchOutput as i32,
            },
            proto::UtxoState {
                height: -1,
                is_confirmed: false,
                state: proto::UtxoStateVariant::NoSuchTx as i32,
            },
        ],
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
pub async fn test_history() -> Result<()> {
    let genesis_pk = hex::decode(GENESIS_PK_HEX)?;
    let client = ChronikClient::new(CHRONIK_URL.to_string())?;
    let history = client
        .script(ScriptType::P2pk, &genesis_pk)
        .history(0)
        .await?;
    assert_eq!(history.num_pages, 1);
    assert_eq!(
        history.txs[0].txid,
        Sha256d::from_hex_be("7455e298a18829d294441acaabcb854e04b2dd609e29c6cc805392271f9c53ea")?
            .as_slice()
            .to_vec(),
    );
    let history_page = client
        .script(ScriptType::P2pk, &genesis_pk)
        .history_with_page_size(0, 20)
        .await?;
    assert_eq!(history, history_page);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
pub async fn test_utxos() -> Result<()> {
    let genesis_pk = hex::decode(GENESIS_PK_HEX)?;
    let client = ChronikClient::new(CHRONIK_URL.to_string())?;
    let utxos = client.script(ScriptType::P2pk, &genesis_pk).utxos().await?;
    assert_eq!(
        utxos,
        vec![proto::ScriptUtxos {
            output_script: [[0x41].as_ref(), &genesis_pk, &[0xac]].concat(),
            utxos: vec![proto::Utxo {
                outpoint: Some(proto::OutPoint {
                    txid: Sha256d::from_hex_be(
                        "7455e298a18829d294441acaabcb854e04b2dd609e29c6cc805392271f9c53ea"
                    )?
                    .as_slice()
                    .to_vec(),
                    out_idx: 1,
                }),
                block_height: 0,
                is_coinbase: true,
                value: 130_000_000,
                slp_meta: None,
                slp_token: None,
                network: proto::Network::Xpi as i32,
            }],
        }],
    );
    Ok(())
}
