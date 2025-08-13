use std::collections::HashMap;

use bitcoinsuite_chronik_client::{
    proto::{self, token_type},
    ChronikClient, ChronikClientError, ScriptType,
};
use bitcoinsuite_core::{Hashed, Sha256d};
use bitcoinsuite_error::Result;

use pretty_assertions::assert_eq;
use reqwest::StatusCode;

const CHRONIK_URL: &str = "https://chronik.pay2stay.com/xec";
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
    let error_msg = "400: Parsing tx failed Invalid length, expected 1 bytes but got 0 bytes";
    assert_eq!(
        response,
        ChronikClientError::ChronikError {
            status_code: StatusCode::BAD_REQUEST,
            error: proto::Error {
                msg: error_msg.to_string(),
            },
            error_msg: error_msg.to_string(),
        },
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
pub async fn test_plugin() -> Result<()> {
    let client = ChronikClient::new(CHRONIK_URL.to_string())?;
    let response = client.plugin("exch_demo", &[0]).history(0).await?;
    assert_eq!(response.txs.len(), 0);
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
    let error_msg = "400: Parsing tx failed Invalid length, expected 1 bytes but got 0 bytes";
    assert_eq!(
        response,
        ChronikClientError::ChronikError {
            status_code: StatusCode::BAD_REQUEST,
            error: proto::Error {
                msg: error_msg.to_string(),
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
        Sha256d::from_hex_be("00000000d1145790a8694403d4063f323d499e655c83426834d4ce2f8dd4a2ee")?;
    let prev_block_hash =
        Sha256d::from_hex_be("000000002a22cfee1f2c846adbd12b3e183d4f97683f85dad08a79780a84bd55")?;
    let expected_height = 170;
    let block = client.block_by_hash(&block_hash).await?;
    assert_eq!(
        block.block_info,
        Some(proto::BlockInfo {
            hash: block_hash.as_slice().to_vec(),
            prev_hash: prev_block_hash.as_slice().to_vec(),
            height: expected_height,
            n_bits: 0x1d00ffff,
            timestamp: 1231731025,
            block_size: 490,
            num_txs: 2,
            num_inputs: 2,
            num_outputs: 3,
            sum_input_sats: 5000000000,
            sum_coinbase_output_sats: 5000000000,
            sum_normal_output_sats: 5000000000,
            sum_burned_sats: 0,
            is_final: true,
        }),
    );
    assert_eq!(client.block_by_height(expected_height).await?, block);
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
        "404: Transaction 0000000000000000000000000000000000000000000000000000000000000000 not \
         found in the index";
    assert_eq!(
        err,
        ChronikClientError::ChronikError {
            status_code: StatusCode::NOT_FOUND,
            error_msg: error_msg.to_string(),
            error: proto::Error {
                msg: error_msg.to_string(),
            },
        },
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
pub async fn test_raw_tx_missing() -> Result<()> {
    let client = ChronikClient::new(CHRONIK_URL.to_string())?;
    let err = client
        .raw_tx(&Sha256d::new([0; 32]))
        .await
        .unwrap_err()
        .downcast::<ChronikClientError>()?;
    let error_msg =
        "404: Transaction 0000000000000000000000000000000000000000000000000000000000000000 not \
         found in the index";
    assert_eq!(
        err,
        ChronikClientError::ChronikError {
            status_code: StatusCode::NOT_FOUND,
            error_msg: error_msg.to_string(),
            error: proto::Error {
                msg: error_msg.to_string(),
            },
        },
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
pub async fn test_tx() -> Result<()> {
    let client = ChronikClient::new(CHRONIK_URL.to_string())?;
    let block_hash =
        Sha256d::from_hex_be("00000000d1145790a8694403d4063f323d499e655c83426834d4ce2f8dd4a2ee")?;
    let txid =
        Sha256d::from_hex_be("f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16")?;
    let actual_tx = client.tx(&txid).await?;
    let expected_tx = proto::Tx {
        txid: txid.as_slice().to_vec(),
        version: 1,
        inputs: vec![proto::TxInput {
            prev_out: Some(proto::OutPoint {
                txid: Sha256d::from_hex_be(
                    "0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9",
                )?
                .as_slice()
                .to_vec(),
                out_idx: 0,
            }),
            input_script: hex::decode(
                "47304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd41022018\
                 1522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901",
            )?,
            output_script: hex::decode(
                "410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84\
                 ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac",
            )?,
            sats: 5_000_000_000,
            sequence_no: 0xffffffff,
            token: None,
            plugins: HashMap::new(),
        }],
        outputs: vec![
            proto::TxOutput {
                sats: 1_000_000_000,
                output_script: hex::decode(
                    "4104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab373\
                     97f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac",
                )?,
                token: None,
                spent_by: Some(proto::SpentBy {
                    txid: Sha256d::from_hex_be(
                        "ea44e97271691990157559d0bdd9959e02790c34db6c006d779e82fa5aee708e",
                    )?
                    .as_slice()
                    .to_vec(),
                    input_idx: 0,
                }),
                plugins: HashMap::new(),
            },
            proto::TxOutput {
                sats: 4_000_000_000,
                output_script: hex::decode(
                    "410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eadd\
                     fb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac",
                )?,
                token: None,
                spent_by: Some(proto::SpentBy {
                    txid: Sha256d::from_hex_be(
                        "a16f3ce4dd5deb92d98ef5cf8afeaf0775ebca408f708b2146c4fb42b41e14be",
                    )?
                    .as_slice()
                    .to_vec(),
                    input_idx: 0,
                }),
                plugins: HashMap::new(),
            },
        ],
        lock_time: 0,
        token_entries: vec![],
        token_failed_parsings: vec![],
        token_status: proto::TokenStatus::NonToken as _,
        block: Some(proto::BlockMetadata {
            height: 170,
            hash: block_hash.as_slice().to_vec(),
            timestamp: 1231731025,
            is_final: true,
        }),
        time_first_seen: 0,
        size: 275,
        is_coinbase: false,
        is_final: true,
    };
    assert_eq!(actual_tx, expected_tx);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
pub async fn test_raw_tx() -> Result<()> {
    let client = ChronikClient::new(CHRONIK_URL.to_string())?;
    let txid =
        Sha256d::from_hex_be("f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16")?;
    let actual_raw_tx = client.raw_tx(&txid).await?;
    let expected_raw_tx =
        "0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd37040000000048473044\
         02204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de48\
         60a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09\
         c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a\
         0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a38\
         2e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3\
         ac00000000";
    assert_eq!(actual_raw_tx.hex(), expected_raw_tx);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
pub async fn test_block_txs() -> Result<()> {
    let client = ChronikClient::new(CHRONIK_URL.to_string())?;
    let block_hash =
        Sha256d::from_hex_be("00000000000053807791091d70e691abff37fc4f8196df306ade8fd8fc40b9e8")?;
    let block_height: i32 = 122740;
    let block_txs_by_hash = client.block_txs_by_hash(&block_hash, 0).await?;
    let block_txs_by_height = client.block_txs_by_height(block_height, 0).await?;
    assert_eq!(block_txs_by_hash, block_txs_by_height);

    let num_txs_in_block: u32 = block_txs_by_hash.num_txs;
    assert_eq!(block_txs_by_hash.num_txs, 64);

    let num_txs_in_page: u32 = block_txs_by_hash.txs.len().try_into().unwrap();
    assert_eq!(
        block_txs_by_hash.num_pages,
        num_txs_in_block / num_txs_in_page + 1
    );

    // Same page size gives the same result
    let page_size: usize = num_txs_in_page.try_into().unwrap();
    let block_txs_by_hash_with_page_size = client
        .block_txs_by_hash_with_page_size(&block_hash, 0, page_size)
        .await?;
    let block_txs_by_height_with_page_size = client
        .block_txs_by_height_with_page_size(block_height, 0, page_size)
        .await?;

    assert_eq!(
        block_txs_by_hash_with_page_size,
        block_txs_by_height_with_page_size
    );
    assert_eq!(block_txs_by_hash_with_page_size, block_txs_by_hash);

    let block_txs_by_hash_with_max_page_size = client
        .block_txs_by_hash_with_page_size(&block_hash, 0, 64)
        .await?;
    assert_eq!(block_txs_by_hash_with_max_page_size.num_pages, 1);
    assert_eq!(block_txs_by_hash_with_max_page_size.num_txs, 64);
    assert_eq!(block_txs_by_hash_with_max_page_size.txs.len(), 64);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
pub async fn test_slpv1_token() -> Result<()> {
    let client = ChronikClient::new(CHRONIK_URL.to_string())?;
    let token_id =
        Sha256d::from_hex_be("0daf200e3418f2df1158efef36fbb507f12928f1fdcf3543703e64e75a4a9073")?;
    let token = client.token(&token_id).await?;
    let block_hash =
        Sha256d::from_hex_be("00000000000000002686aa5ffa8401c7ed67338fb9475561b2fa9817d6571da8")?;
    assert_eq!(
        token,
        proto::TokenInfo {
            token_id: token_id.to_string(),
            token_type: Some(proto::TokenType {
                token_type: Some(token_type::TokenType::Slp(
                    proto::SlpTokenType::Fungible as _
                ))
            }),
            genesis_info: Some(proto::GenesisInfo {
                token_ticker: b"USDR".to_vec(),
                token_name: b"RaiUSD".to_vec(),
                mint_vault_scripthash: vec![],
                url: b"https://www.raiusd.co/etoken".to_vec(),
                hash: vec![],
                data: vec![],
                auth_pubkey: vec![],
                decimals: 4,
            }),
            block: Some(proto::BlockMetadata {
                hash: block_hash.as_slice().to_vec(),
                height: 697721,
                timestamp: 1627783243,
                is_final: true,
            }),
            time_first_seen: token.time_first_seen,
        },
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
pub async fn test_slpv2_token() -> Result<()> {
    let client = ChronikClient::new(CHRONIK_URL.to_string())?;
    let token_id =
        Sha256d::from_hex_be("cdcdcdcdcdc9dda4c92bb1145aa84945c024346ea66fd4b699e344e45df2e145")?;
    let token = client.token(&token_id).await?;
    let block_hash =
        Sha256d::from_hex_be("00000000000000000b7e89959ee52ca1cd691e1fc3b4891c1888f84261c83e73")?;
    assert_eq!(
        token,
        proto::TokenInfo {
            token_id: token_id.to_string(),
            token_type: Some(proto::TokenType {
                token_type: Some(token_type::TokenType::Alp(
                    proto::AlpTokenType::Standard as _
                ))
            }),
            genesis_info: Some(proto::GenesisInfo {
                token_ticker: b"CRD".to_vec(),
                token_name: b"Credo In Unum Deo".to_vec(),
                mint_vault_scripthash: vec![],
                url: b"https://crd.network/token".to_vec(),
                hash: vec![],
                data: vec![],
                auth_pubkey: hex::decode(
                    "0334b744e6338ad438c92900c0ed1869c3fd2c0f35a4a9b97a88447b6e2b145f10"
                )
                .unwrap(),
                decimals: 4,
            }),
            block: Some(proto::BlockMetadata {
                hash: block_hash.as_slice().to_vec(),
                height: 795680,
                timestamp: 1686305735,
                is_final: true,
            }),
            time_first_seen: token.time_first_seen,
        },
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
pub async fn test_history() -> Result<()> {
    let genesis_pk = hex::decode(GENESIS_PK_HEX)?;
    let client = ChronikClient::new(CHRONIK_URL.to_string())?;
    let history = client
        .script(ScriptType::P2pk, &genesis_pk)
        .confirmed_txs(0)
        .await?;
    assert_eq!(history.num_pages, 1);
    assert_eq!(
        history.txs[0].txid,
        Sha256d::from_hex_be("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b")?
            .as_slice()
            .to_vec(),
    );
    Ok(())
}
