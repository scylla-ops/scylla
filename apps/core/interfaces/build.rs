use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);

    let workspace_root = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let proto_dir = workspace_root.join("libs/protocol/proto");

    let proto_files = vec![
        "common.proto",
        "user.proto",
        "auth.proto",
        "orchestrator.proto",
        "pipeline.proto",
        "pipeline_def.proto",
        "job.proto",
        "job_def.proto",
        "organization.proto",
        "project.proto",
    ];

    let protos: Vec<PathBuf> = proto_files.iter().map(|f| proto_dir.join(f)).collect();

    for proto in &protos {
        if !proto.exists() {
            return Err(format!("ERREUR : Fichier introuvable à l'adresse : {:?}", proto).into());
        }
        println!("cargo:rerun-if-changed={}", proto.display());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("services_descriptor.bin"))
        .compile_protos(&protos, &[proto_dir])?;

    println!("cargo:rustc-env=OUT_DIR={}", out_dir.display());
    Ok(())
}
