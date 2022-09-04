use bitcoinsuite_bitcoind::instance::{BitcoindChain, BitcoindConf, BitcoindInstance};
use bitcoinsuite_core::{
    build_bitcoin_block, build_bitcoin_coinbase, ecc::PubKey, BitcoinCode, Hashed, Script, Sha256d,
};
use bitcoinsuite_test_utils::bin_folder;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_xec_blocks() -> Result<(), Box<dyn std::error::Error>> {
    let bitcoind_conf = BitcoindConf::from_chain_regtest(bin_folder(), BitcoindChain::XEC, vec![])?;

    let mut bitcoind = BitcoindInstance::setup(bitcoind_conf)?;
    bitcoind.wait_for_ready()?;

    let prev_block_hash = bitcoind.cmd_string("getbestblockhash", &[])?;
    let mut prev_block_hash = Sha256d::from_hex_be(&prev_block_hash)?;

    let timestamp = 1_600_000_000;

    for height in 1_i32..1000 {
        let script = Script::p2pk(&PubKey::new_unchecked([0x03; 33]));
        let coinbase = build_bitcoin_coinbase(height, script);
        let block = build_bitcoin_block(
            prev_block_hash,
            timestamp + height as u32,
            coinbase.hashed(),
            vec![],
        );
        prev_block_hash = block.header.calc_hash();

        let result = bitcoind.cmd_string("submitblock", &[&block.ser().hex()])?;
        assert_eq!(result, "");
    }

    Ok(())
}
