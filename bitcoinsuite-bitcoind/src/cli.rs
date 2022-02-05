use std::{
    ffi::OsString,
    path::PathBuf,
    process::{Command, Output},
};

use bitcoinsuite_error::{Result, WrapErr};

use crate::BitcoindError;

#[derive(Debug, Clone)]
pub struct BitcoinCli {
    pub bitcoincli_path: PathBuf,
    pub datadir_arg: OsString,
}

impl BitcoinCli {
    pub fn cmd_output(&self, cmd: &str, args: &[&str]) -> Result<Output> {
        Command::new(&self.bitcoincli_path)
            .arg(&self.datadir_arg)
            .arg(cmd)
            .args(args)
            .output()
            .wrap_err(BitcoindError::Client)
    }

    pub fn cmd_string(&self, cmd: &str, args: &[&str]) -> Result<String> {
        let output = self.cmd_output(cmd, args)?;
        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?
                .trim_end_matches('\n')
                .to_string())
        } else {
            Err(BitcoindError::JsonRpc(String::from_utf8(output.stderr)?).into())
        }
    }

    pub fn cmd_json(&self, cmd: &str, args: &[&str]) -> Result<json::JsonValue> {
        Ok(json::parse(&self.cmd_string(cmd, args)?)?)
    }
}
