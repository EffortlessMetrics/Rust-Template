fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = std::path::Path::new("../../specs/proto/task.proto");
    tonic_prost_build::compile_protos(proto_file)?;
    Ok(())
}
