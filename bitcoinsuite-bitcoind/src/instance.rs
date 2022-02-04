use std::{
    ffi::OsString,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, Output},
    str::FromStr,
    time::Duration,
};

use bitcoinsuite_core::Net;
use bitcoinsuite_error::{Result, WrapErr};
use bitcoinsuite_test_utils::pick_ports;
use tempdir::TempDir;

use crate::{client::BitcoindClient, BitcoindError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitcoindChain {
    XEC,
    BCH,
    XPI,
}

pub struct BitcoindConf {
    bitcoind_path: PathBuf,
    bitcoincli_path: PathBuf,
    additional_args: Vec<OsString>,
    p2p_port: u16,
    rpc_port: u16,
    net: Net,
    chain: BitcoindChain,
}

pub struct BitcoindInstance {
    conf: BitcoindConf,
    instance_dir: PathBuf,
    datadir_arg: OsString,
    bitcoind_child: Child,
    client: BitcoindClient,
}

impl BitcoindConf {
    pub fn from_chain_regtest(
        bin_folder: impl AsRef<Path>,
        chain: BitcoindChain,
        additional_args: Vec<OsString>,
    ) -> Result<Self> {
        Self::new(bin_folder, chain, Net::Regtest, additional_args)
    }

    pub fn new(
        bin_folder: impl AsRef<Path>,
        chain: BitcoindChain,
        net: Net,
        additional_args: Vec<OsString>,
    ) -> Result<Self> {
        let ports = pick_ports(2)?;
        let bin_folder = bin_folder.as_ref();
        let bin_folder = match chain {
            BitcoindChain::XEC => bin_folder.join("bitcoin-abc").join("bin"),
            BitcoindChain::BCH => bin_folder.join("bitcoin-cash-node").join("bin"),
            BitcoindChain::XPI => bin_folder.join("lotusd").join("bin"),
        };
        let (bitcoind_path, bitcoincli_path) = match chain {
            BitcoindChain::XPI => (bin_folder.join("lotusd"), bin_folder.join("lotus-cli")),
            _ => (bin_folder.join("bitcoind"), bin_folder.join("bitcoin-cli")),
        };
        Ok(BitcoindConf {
            bitcoind_path,
            bitcoincli_path,
            additional_args,
            p2p_port: ports[0],
            rpc_port: ports[1],
            net,
            chain,
        })
    }
}

impl BitcoindInstance {
    pub fn setup(conf: BitcoindConf) -> Result<Self> {
        let instance_dir = TempDir::new("bitcoind_test_dir")
            .wrap_err(BitcoindError::TestInstance)?
            .into_path();
        let datadir = instance_dir.join("datadir");
        std::fs::create_dir(&datadir).wrap_err(BitcoindError::TestInstance)?;
        Self::start(instance_dir, datadir, conf)
    }

    pub fn start(
        instance_dir: PathBuf,
        datadir: impl AsRef<Path>,
        conf: BitcoindConf,
    ) -> Result<Self> {
        let mut datadir_arg = OsString::from_str("-datadir=").unwrap();
        datadir_arg.push(datadir.as_ref().as_os_str());
        let datadir = datadir.as_ref();
        println!("{}", datadir.to_str().unwrap());
        let stdout =
            File::create(instance_dir.join("stdout.txt")).wrap_err(BitcoindError::TestInstance)?;
        let stderr =
            File::create(instance_dir.join("stderr.txt")).wrap_err(BitcoindError::TestInstance)?;
        let bitcoin_conf_str = format!(
            "\
{net_line}
server=1
rpcuser=user
rpcpassword=pass
{net_section_header}
port={p2p_port}
rpcport={rpc_port}
",
            net_line = net_conf_line(conf.net),
            net_section_header = net_conf_section_header(conf.net),
            p2p_port = conf.p2p_port,
            rpc_port = conf.rpc_port
        );
        let conf_path = match conf.chain {
            BitcoindChain::XPI => datadir.join("lotus.conf"),
            _ => datadir.join("bitcoin.conf"),
        };
        {
            let mut bitcoin_conf = File::create(conf_path).wrap_err(BitcoindError::TestInstance)?;
            bitcoin_conf
                .write_all(bitcoin_conf_str.as_bytes())
                .wrap_err(BitcoindError::TestInstance)?;
            bitcoin_conf.flush().wrap_err(BitcoindError::TestInstance)?;
        }
        let mut datadir_arg = OsString::from_str("-datadir=").unwrap();
        datadir_arg.push(datadir.as_os_str());
        let bitcoind_child = Command::new(&conf.bitcoind_path)
            .arg(&datadir_arg)
            .args(&conf.additional_args)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .wrap_err(BitcoindError::TestInstance)?;
        let client = BitcoindClient {
            datadir_arg: datadir_arg.clone(),
            bitcoincli_path: conf.bitcoincli_path.clone(),
        };
        Ok(BitcoindInstance {
            conf,
            instance_dir,
            datadir_arg,
            bitcoind_child,
            client,
        })
    }

    pub fn client(&self) -> &BitcoindClient {
        &self.client
    }

    fn shutdown_bitcoind(&mut self) -> Result<()> {
        self.bitcoind_child
            .kill()
            .wrap_err(BitcoindError::TestInstance)?;
        self.bitcoind_child
            .wait()
            .wrap_err(BitcoindError::TestInstance)?;
        Ok(())
    }

    pub fn restart_with_args(&mut self, args: &[OsString]) -> Result<()> {
        self.shutdown_bitcoind()?;
        let stdout = File::create(self.instance_dir.join("stdout1.txt"))
            .wrap_err(BitcoindError::TestInstance)?;
        let stderr = File::create(self.instance_dir.join("stderr1.txt"))
            .wrap_err(BitcoindError::TestInstance)?;
        let bitcoind_child = Command::new(&self.conf.bitcoind_path)
            .arg(&self.datadir_arg)
            .args(args)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .wrap_err(BitcoindError::TestInstance)?;
        self.bitcoind_child = bitcoind_child;
        Ok(())
    }

    pub fn cmd_output(&self, cmd: &str, args: &[&str]) -> Result<Output> {
        self.client.cmd_output(cmd, args)
    }

    pub fn cmd_string(&self, cmd: &str, args: &[&str]) -> Result<String> {
        self.client.cmd_string(cmd, args)
    }

    pub fn cmd_json(&self, cmd: &str, args: &[&str]) -> Result<json::JsonValue> {
        self.client.cmd_json(cmd, args)
    }

    fn _ensure_bitcoind(&mut self) -> Result<()> {
        if self
            .bitcoind_child
            .try_wait()
            .wrap_err(BitcoindError::TestInstance)?
            .is_some()
        {
            return Err(BitcoindError::BitcoindExited.into());
        }
        Ok(())
    }

    pub fn p2p_port(&self) -> u16 {
        self.conf.p2p_port
    }

    pub fn wait_for_ready(&mut self) -> Result<()> {
        for _ in 0..100 {
            self._ensure_bitcoind()?;
            std::thread::sleep(Duration::from_millis(50));
            let output = self.cmd_output("getblockcount", &[])?;
            if output.status.success() {
                return Ok(());
            }
        }
        Err(BitcoindError::Timeout("bitcoind".into()).into())
    }

    pub fn cleanup(&self) -> Result<()> {
        std::fs::remove_dir_all(&self.instance_dir).wrap_err(BitcoindError::TestInstance)
    }
}

impl Drop for BitcoindInstance {
    fn drop(&mut self) {
        if let Ok(None) = self.bitcoind_child.try_wait() {
            if let Err(err) = self.shutdown_bitcoind() {
                eprintln!("Failed to shut down bitcoind: {}", err);
            }
        }
    }
}

fn net_conf_line(net: Net) -> &'static str {
    match net {
        Net::Mainnet => "",
        Net::Regtest => "regtest=1",
    }
}

fn net_conf_section_header(net: Net) -> &'static str {
    match net {
        Net::Mainnet => "",
        Net::Regtest => "[regtest]",
    }
}
