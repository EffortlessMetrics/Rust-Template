fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("PROTOC", protobuf_src::protoc());
    }
    let proto_file = std::path::Path::new("../../specs/proto/task/v1/task.proto");
    tonic_prost_build::compile_protos(proto_file)?;
    Ok(())
}
