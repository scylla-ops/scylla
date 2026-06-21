use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let proto_dir = manifest_dir.join("proto");

    let protos = [
        "common.proto",
        "user.proto",
        "auth.proto",
        "registration.proto",
        "invitation.proto",
        "oauth.proto",
        "organization.proto",
        "project.proto",
        "permission.proto",
        "pipeline.proto",
        "secret.proto",
        "job.proto",
        "app.proto",
        "agent.proto",
        "agent_admin.proto",
        "trigger.proto",
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
