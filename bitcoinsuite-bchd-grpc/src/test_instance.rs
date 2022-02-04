use std::{
    ffi::OsString,
    io::Write,
    path::PathBuf,
    process::{Child, Command},
    str::FromStr,
};

use bitcoinsuite_test_utils::{bin_folder, pick_ports};
use tempdir::TempDir;
use tonic::transport::Channel;

use crate::{
    bchd_grpc::{bchrpc_client::BchrpcClient, GetTransactionRequest, GetTransactionResponse},
    connect_bchd,
    error::Result,
    BchdError,
};

#[derive(Debug, Clone)]
pub struct BchdTestConf {
    bchd_path: PathBuf,
    additional_args: Vec<OsString>,
    connect_port: u16,
    rpc_port: u16,
    grpc_port: u16,
    rpc_user: String,
    rpc_pass: String,
}
#[derive(Debug)]
pub struct BchdTestInstance {
    instance_dir: PathBuf,
    client: BchrpcClient<Channel>,
    bchd_child: Child,
}
impl BchdTestConf {
    pub fn from_env(connect_port: u16, additional_args: Vec<OsString>) -> Result<Self> {
        let ports = pick_ports(2)?;
        Ok(BchdTestConf {
            bchd_path: bin_folder().join("bchd").join("bchd"),
            additional_args,
            connect_port,
            rpc_port: ports[0],
            grpc_port: ports[1],
            rpc_user: "user".to_string(),
            rpc_pass: "pass".to_string(),
        })
    }
}

impl BchdTestInstance {
    pub async fn setup(conf: BchdTestConf) -> Result<Self> {
        let instance_dir = TempDir::new("bchd_test_dir")
            .map_err(BchdError::TestInstanceIo)?
            .into_path();
        let datadir = instance_dir.join("datadir");
        let certdir = instance_dir.join("certdir");
        let logdir = instance_dir.join("logdir");
        let cert_path = certdir.join("localhost.crt");
        let conf_file_path = instance_dir.join("bchd.conf");
        println!("Running BCHD in {}", instance_dir.to_str().unwrap());
        std::fs::create_dir(&datadir).map_err(BchdError::TestInstanceIo)?;
        std::fs::create_dir(&certdir).map_err(BchdError::TestInstanceIo)?;
        std::fs::create_dir(&logdir).map_err(BchdError::TestInstanceIo)?;
        let stdout = std::fs::File::create(instance_dir.join("stdout.txt"))
            .map_err(BchdError::TestInstanceIo)?;
        let stderr = std::fs::File::create(instance_dir.join("stderr.txt"))
            .map_err(BchdError::TestInstanceIo)?;
        let bitcoin_conf_str = format!(
            "\
[Application Options]
regtest=1
connect=127.0.0.1:{connect_port}
nolisten=1
rpclisten=127.0.0.1:{rpc_port}
grpclisten=127.0.0.1:{grpc_port}
rpcuser={rpc_user}
rpcpass={rpc_pass}
rpccert={cert_path}
rpckey={certdir}/localhost.key
rejectnonstd=0
norelaypriority=1
minrelaytxfee=0.00001
debuglevel=trace
txindex=1
addrindex=1
slpindex=1",
            connect_port = conf.connect_port,
            rpc_port = conf.rpc_port,
            grpc_port = conf.grpc_port,
            rpc_user = conf.rpc_user,
            rpc_pass = conf.rpc_pass,
            cert_path = cert_path.to_string_lossy(),
            certdir = certdir.to_string_lossy(),
        );
        {
            let mut bchd_conf =
                std::fs::File::create(&conf_file_path).map_err(BchdError::TestInstanceIo)?;
            bchd_conf
                .write_all(bitcoin_conf_str.as_bytes())
                .map_err(BchdError::TestInstanceIo)?;
            bchd_conf.flush().map_err(BchdError::TestInstanceIo)?;
        }
        let mut configfile_arg = OsString::from_str("--configfile=").unwrap();
        configfile_arg.push(conf_file_path.as_os_str());
        let mut datadir_arg = OsString::from_str("--datadir=").unwrap();
        datadir_arg.push(datadir.as_os_str());
        let mut logdir_arg = OsString::from_str("--logdir=").unwrap();
        logdir_arg.push(logdir.as_os_str());
        let bchd_child = Command::new(&conf.bchd_path)
            .arg(&configfile_arg)
            .arg(&datadir_arg)
            .arg(&logdir_arg)
            .args(&conf.additional_args)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .map_err(BchdError::TestInstanceIo)?;
        let mut attempts: i32 = 0;
        let client = loop {
            match connect_bchd(format!("http://localhost:{}", conf.grpc_port), &cert_path).await {
                Ok(client) => break client,
                Err(_) if attempts < 10 => {
                    attempts += 1;
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(err) => return Err(err),
            }
        };
        Ok(BchdTestInstance {
            instance_dir,
            client,
            bchd_child,
        })
    }

    pub fn client(&mut self) -> &mut BchrpcClient<Channel> {
        &mut self.client
    }

    pub async fn wait_for_tx(
        &mut self,
        txid: &[u8],
    ) -> std::result::Result<Option<GetTransactionResponse>, tonic::Status> {
        let mut attempts = 0;
        loop {
            match self
                .client
                .get_transaction(GetTransactionRequest {
                    hash: txid.to_vec(),
                    include_token_metadata: true,
                })
                .await
            {
                Ok(tx) => return Ok(Some(tx.into_inner())),
                Err(_) if attempts < 50 => attempts += 1,
                Err(_) => return Ok(None),
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    fn shutdown_bchd(&mut self) -> Result<()> {
        self.bchd_child.kill().map_err(BchdError::TestInstanceIo)?;
        self.bchd_child.wait().map_err(BchdError::TestInstanceIo)?;
        Ok(())
    }

    pub fn cleanup(&self) -> Result<()> {
        std::fs::remove_dir_all(&self.instance_dir).map_err(BchdError::TestInstanceIo)?;
        Ok(())
    }
}

impl Drop for BchdTestInstance {
    fn drop(&mut self) {
        if let Ok(None) = self.bchd_child.try_wait() {
            if let Err(err) = self.shutdown_bchd() {
                eprintln!("Failed to shut down bitcoind: {}", err);
            }
        }
    }
}
