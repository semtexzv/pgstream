use std::io::Result;
fn main() -> Result<()> {
    prost_build::Config::new()
        .out_dir("src")
        .compile_protos(&["./proto/pg_logicaldec.proto"], &["./proto"])?;
    Ok(())
}
