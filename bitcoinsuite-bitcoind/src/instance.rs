use std::{
    ffi::OsString,
    fs::File,
    io::{BufRead, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Output},
    str::FromStr,
    time::Duration,
};

use bitcoinsuite_core::Net;
use bitcoinsuite_error::{Result, WrapErr};
use bitcoinsuite_test_utils::pick_ports;
use rev_buf_reader::RevBufReader;
use tempdir::TempDir;

use crate::{
    cli::BitcoinCli,
    rpc_client::{BitcoindRpcClient, BitcoindRpcClientConf},
    BitcoindError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitcoindChain {
    XEC,
    BCH,
    XPI,
}

#[derive(Debug, Clone)]
pub struct BitcoindConf {
    pub bitcoind_path: PathBuf,
    pub bitcoincli_path: PathBuf,
    pub additional_args: Vec<OsString>,
    pub p2p_port: u16,
    pub rpc_port: u16,
    pub chronik_port: Option<u16>,
    pub net: Net,
    pub chain: BitcoindChain,
}

#[derive(Debug)]
pub struct BitcoindInstance {
    conf: BitcoindConf,
    instance_dir: PathBuf,
    datadir_arg: OsString,
    bitcoind_child: Child,
    cli: BitcoinCli,
    client: BitcoindRpcClient,
    chronik_url: String,
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
        let ports = pick_ports(3)?;
        let bin_folder = bin_folder.as_ref();
        let (bin_folder, chronik_port) = match chain {
            BitcoindChain::XEC => (bin_folder.join("bitcoin-abc").join("bin"), Some(ports[2])),
            BitcoindChain::BCH => (bin_folder.join("bitcoin-cash-node").join("bin"), None),
            BitcoindChain::XPI => (bin_folder.join("lotusd").join("bin"), None),
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
            chronik_port,
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
        let rpc_user = "user";
        let rpc_pass = "pass";
        let (chronik_url, chronik_conf);
        match conf.chronik_port {
            Some(port) => {
                let chronik_host = format!("127.0.0.1:{port}");
                chronik_url = format!("http://{chronik_host}");
                chronik_conf = format!(
                    "\
chronik=1
chronikbind={chronik_host}"
                );
            }
            None => {
                chronik_url = String::new();
                chronik_conf = String::new();
            }
        }
        let bitcoin_conf_str = format!(
            "\
{net_line}
server=1
rpcuser={rpc_user}
rpcpassword={rpc_pass}
{net_section_header}
port={p2p_port}
rpcport={rpc_port}
{chronik_conf}
",
            net_line = net_conf_line(conf.net),
            net_section_header = net_conf_section_header(conf.net),
            p2p_port = conf.p2p_port,
            rpc_port = conf.rpc_port,
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
        let cli = BitcoinCli {
            datadir_arg: datadir_arg.clone(),
            bitcoincli_path: conf.bitcoincli_path.clone(),
        };
        let client = BitcoindRpcClient::new(BitcoindRpcClientConf {
            url: format!("http://127.0.0.1:{}", conf.rpc_port),
            rpc_user: rpc_user.to_string(),
            rpc_pass: rpc_pass.to_string(),
        });
        Ok(BitcoindInstance {
            conf,
            instance_dir,
            datadir_arg,
            bitcoind_child,
            cli,
            client,
            chronik_url,
        })
    }

    pub fn cli(&self) -> &BitcoinCli {
        &self.cli
    }

    pub fn rpc_client(&self) -> &BitcoindRpcClient {
        &self.client
    }

    pub fn chronik_url(&self) -> &str {
        &self.chronik_url
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
        self.cli.cmd_output(cmd, args)
    }

    pub fn cmd_string(&self, cmd: &str, args: &[&str]) -> Result<String> {
        self.cli.cmd_string(cmd, args)
    }

    pub fn cmd_json(&self, cmd: &str, args: &[&str]) -> Result<json::JsonValue> {
        self.cli.cmd_json(cmd, args)
    }

    fn _ensure_bitcoind(&mut self) -> Result<()> {
        if self
            .bitcoind_child
            .try_wait()
            .wrap_err(BitcoindError::TestInstance)?
            .is_some()
        {
            return Err(BitcoindError::BitcoindExited {
                stderr: self
                    .read_stderr()
                    .unwrap_or_else(|err| format!("Failed opening stderr.txt: {err}")),
                debug_log_tail: self
                    .read_debug_log_tail()
                    .unwrap_or_else(|err| format!("Failed opening debug.log: {err}")),
            }
            .into());
        }
        Ok(())
    }

    pub fn p2p_port(&self) -> u16 {
        self.conf.p2p_port
    }

    pub fn chronik_port(&self) -> Option<u16> {
        self.conf.chronik_port
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

    pub fn terminate(&mut self) -> Result<()> {
        Command::new("kill")
            .args(["-s", "TERM", &self.bitcoind_child.id().to_string()])
            .spawn()?
            .wait()?;
        self.bitcoind_child.wait()?;
        Ok(())
    }

    pub fn cleanup(&self) -> Result<()> {
        std::fs::remove_dir_all(&self.instance_dir).wrap_err(BitcoindError::TestInstance)
    }

    fn read_stderr(&self) -> Result<String, std::io::Error> {
        let stderr_path = self.instance_dir.join("stderr.txt");
        if !stderr_path.exists() {
            return Ok("".to_string());
        }
        let mut file = File::open(stderr_path)?;
        let mut stderr = String::new();
        file.read_to_string(&mut stderr)?;
        Ok(stderr)
    }

    fn read_debug_log_tail(&self) -> Result<String, std::io::Error> {
        let debug_log_path = self.instance_dir.join("datadir").join("debug.log");
        if !debug_log_path.exists() {
            return Ok("".to_string());
        }
        let mut file = File::open(debug_log_path)?;
        let rev_reader = RevBufReader::new(&mut file);
        let lines = rev_reader.lines().take(20).collect::<Vec<_>>();
        let mut debug_log = String::new();
        for line in lines.into_iter() {
            debug_log.push_str(&line?);
            debug_log.push('\n');
        }
        Ok(debug_log)
    }
}

impl Drop for BitcoindInstance {
    fn drop(&mut self) {
        if let Ok(None) = self.bitcoind_child.try_wait() {
            if let Err(err) = self.shutdown_bitcoind() {
                eprintln!("Failed to shut down bitcoind: {err}");
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
