use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = &[
        "proto/common.proto",
        "proto/user.proto",
        "proto/auth.proto",
        "proto/orchestrator.proto",
        "proto/pipeline.proto",
        "proto/job.proto",
        "proto/organization.proto",
        "proto/project.proto",
    ];

    for proto in protos {
        println!("cargo:rerun-if-changed={}", proto);
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("services_descriptor.bin"))
        .compile_protos(protos, &["proto/"])?;
    Ok(())
}
