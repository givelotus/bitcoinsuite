use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["proto/chronik.proto"], &["proto/"])?;
    println!("cargo:rerun-if-changed=proto/chronik.proto");
    Ok(())
}
