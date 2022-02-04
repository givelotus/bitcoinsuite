use bitcoinsuite_bchd_grpc::bchd_grpc::GetBlockchainInfoRequest;
use bitcoinsuite_core::Script;
use bitcoinsuite_error::Result;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_setup_xec_chain() -> Result<()> {
    let redeem_script = Script::from_slice(&[0x51]);
    let (bitcoind, mut bchd, utxos) =
        test_utils_blockchain::setup_xec_chain(10, &redeem_script).await?;
    assert_eq!(bitcoind.cmd_string("getblockcount", &[])?, "110");
    let response = bchd
        .client()
        .get_blockchain_info(GetBlockchainInfoRequest {})
        .await?
        .into_inner();
    assert_eq!(response.best_height, 110);
    assert_eq!(utxos.len(), 10);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_setup_bch_chain() -> Result<()> {
    let redeem_script = Script::from_slice(&[0x51]);
    let (bitcoind, mut bchd, utxos) =
        test_utils_blockchain::setup_bch_chain(10, &redeem_script).await?;
    assert_eq!(bitcoind.cmd_string("getblockcount", &[])?, "110");
    let response = bchd
        .client()
        .get_blockchain_info(GetBlockchainInfoRequest {})
        .await?
        .into_inner();
    assert_eq!(response.best_height, 110);
    assert_eq!(utxos.len(), 10);
    Ok(())
}
