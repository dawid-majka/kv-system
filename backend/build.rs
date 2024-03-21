fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(&["../proto/key_value.proto"], &["../proto"])?;
    Ok(())
}
