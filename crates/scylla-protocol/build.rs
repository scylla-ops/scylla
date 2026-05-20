use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let proto_dir = manifest_dir.join("proto");

    let protos = [
        "common.proto",
        "config.proto",
        "user.proto",
        "auth.proto",
        "registration.proto",
        "organization.proto",
        "project.proto",
        "permission.proto",
        "pipeline.proto",
        "job.proto",
        "agent.proto",
    ]
    .map(|f| proto_dir.join(f));

    for proto in &protos {
        println!("cargo:rerun-if-changed={}", proto.display());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("services_descriptor.bin"))
        .compile_protos(&protos, &[proto_dir])?;

    Ok(())
}
