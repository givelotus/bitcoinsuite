use std::{
    ffi::OsString,
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, Output},
    str::FromStr,
    time::Duration,
};

use bitcoinsuite_test_utils::pick_ports;
use tempdir::TempDir;

use crate::{error::Result, BitcoindError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitcoindChain {
    XEC,
    BCH,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitcoindNet {
    Mainnet,
    Regtest,
}

pub struct BitcoindConf {
    bitcoind_path: PathBuf,
    bitcoincli_path: PathBuf,
    additional_args: Vec<OsString>,
    p2p_port: u16,
    rpc_port: u16,
    net: BitcoindNet,
}

pub struct BitcoindInstance {
    conf: BitcoindConf,
    instance_dir: PathBuf,
    datadir_arg: OsString,
    bitcoind_child: Child,
}

impl BitcoindConf {
    pub fn from_chain_regtest(
        bin_folder: impl AsRef<Path>,
        chain: BitcoindChain,
        additional_args: Vec<OsString>,
    ) -> Result<Self> {
        Self::new(bin_folder, chain, BitcoindNet::Regtest, additional_args)
    }

    pub fn new(
        bin_folder: impl AsRef<Path>,
        chain: BitcoindChain,
        net: BitcoindNet,
        additional_args: Vec<OsString>,
    ) -> Result<Self> {
        let ports = pick_ports(2)?;
        let bin_folder = bin_folder.as_ref();
        let bin_folder = match chain {
            BitcoindChain::XEC => bin_folder.join("bitcoin-abc").join("bin"),
            BitcoindChain::BCH => bin_folder.join("bitcoin-cash-node").join("bin"),
        };
        let bitcoind_path = bin_folder.join("bitcoind");
        let bitcoincli_path = bin_folder.join("bitcoin-cli");
        Ok(BitcoindConf {
            bitcoind_path,
            bitcoincli_path,
            additional_args,
            p2p_port: ports[0],
            rpc_port: ports[1],
            net,
        })
    }
}

impl BitcoindInstance {
    pub fn setup(conf: BitcoindConf) -> Result<Self> {
        let instance_dir = TempDir::new("bitcoind_test_dir")
            .map_err(BitcoindError::TestInstance)?
            .into_path();
        let datadir = instance_dir.join("datadir");
        println!("{}", datadir.to_str().unwrap());
        std::fs::create_dir(&datadir).map_err(BitcoindError::TestInstance)?;
        let stdout = std::fs::File::create(instance_dir.join("stdout.txt"))
            .map_err(BitcoindError::TestInstance)?;
        let stderr = std::fs::File::create(instance_dir.join("stderr.txt"))
            .map_err(BitcoindError::TestInstance)?;
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
            net_line = conf.net.conf_line(),
            net_section_header = conf.net.conf_section_header(),
            p2p_port = conf.p2p_port,
            rpc_port = conf.rpc_port
        );
        {
            let mut bitcoin_conf = std::fs::File::create(datadir.join("bitcoin.conf"))
                .map_err(BitcoindError::TestInstance)?;
            bitcoin_conf
                .write_all(bitcoin_conf_str.as_bytes())
                .map_err(BitcoindError::TestInstance)?;
            bitcoin_conf.flush().map_err(BitcoindError::TestInstance)?;
        }
        let mut datadir_arg = OsString::from_str("-datadir=").unwrap();
        datadir_arg.push(datadir.as_os_str());
        let bitcoind_child = Command::new(&conf.bitcoind_path)
            .arg(&datadir_arg)
            .args(&conf.additional_args)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .map_err(BitcoindError::TestInstance)?;
        Ok(BitcoindInstance {
            conf,
            instance_dir,
            datadir_arg,
            bitcoind_child,
        })
    }

    fn shutdown_bitcoind(&mut self) -> Result<()> {
        self.bitcoind_child
            .kill()
            .map_err(BitcoindError::TestInstance)?;
        self.bitcoind_child
            .wait()
            .map_err(BitcoindError::TestInstance)?;
        Ok(())
    }

    pub fn restart_with_args(&mut self, args: &[OsString]) -> Result<()> {
        self.shutdown_bitcoind()?;
        let stdout = std::fs::File::create(self.instance_dir.join("stdout1.txt"))
            .map_err(BitcoindError::TestInstance)?;
        let stderr = std::fs::File::create(self.instance_dir.join("stderr1.txt"))
            .map_err(BitcoindError::TestInstance)?;
        let bitcoind_child = Command::new(&self.conf.bitcoind_path)
            .arg(&self.datadir_arg)
            .args(args)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .map_err(BitcoindError::TestInstance)?;
        self.bitcoind_child = bitcoind_child;
        Ok(())
    }

    pub fn cmd_output(&self, cmd: &str, args: &[&str]) -> Result<Output> {
        Command::new(&self.conf.bitcoincli_path)
            .arg(&self.datadir_arg)
            .arg(cmd)
            .args(args)
            .output()
            .map_err(BitcoindError::TestInstance)
    }

    pub fn cmd_string(&self, cmd: &str, args: &[&str]) -> Result<String> {
        let output = self.cmd_output(cmd, args)?;
        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?
                .trim_end_matches('\n')
                .to_string())
        } else {
            Err(BitcoindError::JsonRpc(String::from_utf8(output.stderr)?))
        }
    }

    pub fn cmd_json(&self, cmd: &str, args: &[&str]) -> Result<json::JsonValue> {
        Ok(json::parse(&self.cmd_string(cmd, args)?)?)
    }

    fn _ensure_bitcoind(&mut self) -> Result<()> {
        if self
            .bitcoind_child
            .try_wait()
            .map_err(BitcoindError::TestInstance)?
            .is_some()
        {
            return Err(BitcoindError::BitcoindExited);
        }
        Ok(())
    }

    pub fn p2p_port(&self) -> u16 {
        self.conf.p2p_port
    }

    pub fn wait_for_ready(&mut self) -> Result<()> {
        for _ in 0..40 {
            self._ensure_bitcoind()?;
            std::thread::sleep(Duration::from_millis(50));
            let output = self.cmd_output("getblockcount", &[])?;
            if output.status.success() {
                return Ok(());
            }
        }
        Err(BitcoindError::Timeout("bitcoind".into()))
    }

    pub fn cleanup(&self) -> Result<()> {
        std::fs::remove_dir_all(&self.instance_dir).map_err(BitcoindError::TestInstance)
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

impl BitcoindNet {
    fn conf_line(&self) -> &'static str {
        match self {
            BitcoindNet::Mainnet => "",
            BitcoindNet::Regtest => "regtest=1",
        }
    }

    fn conf_section_header(&self) -> &'static str {
        match self {
            BitcoindNet::Mainnet => "",
            BitcoindNet::Regtest => "[regtest]",
        }
    }
}
