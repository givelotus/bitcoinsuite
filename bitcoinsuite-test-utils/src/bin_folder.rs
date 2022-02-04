use std::path::PathBuf;

pub fn bin_folder() -> PathBuf {
    std::env::var_os("BITCOINSUITE_BIN_DIR")
        .expect("Missing BITCOINSUITE_BIN_DIR environment variable")
        .into()
}
