fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("./protos/currency.proto")?;
    tonic_build::compile_protos("./protos/kraken.proto")?;
    tonic_build::compile_protos("./protos/blocks.proto")?;
    Ok(())
}
