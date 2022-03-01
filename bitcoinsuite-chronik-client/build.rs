use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["proto/chronik.proto"], &["proto/"])?;
    Ok(())
}
