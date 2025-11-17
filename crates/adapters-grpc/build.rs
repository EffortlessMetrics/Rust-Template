fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../../../specs/proto/task.proto")?;
    Ok(())
}
