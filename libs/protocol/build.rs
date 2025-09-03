use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/counter.proto");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("services_descriptor.bin"))
        .compile_protos(&["proto/user.proto", "proto/auth.proto"], &["proto/"])?;
    Ok(())
}
