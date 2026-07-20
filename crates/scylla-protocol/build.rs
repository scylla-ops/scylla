use std::env;
use std::path::PathBuf;

/// Every .proto in the workspace, listed explicitly rather than globbed: the
/// build stays deterministic and a stray scratch file cannot break everyone.
/// Paths are relative to the single include root (`proto/`) and mirror their
/// package exactly — `scylla.job.v1` lives in `scylla/job/v1/`.
const PROTOS: &[&str] = &[
    "scylla/common/v1/common.proto",
    "scylla/exec/v1/step.proto",
    "scylla/auth/v1/auth.proto",
    "scylla/registration/v1/registration.proto",
    "scylla/invitation/v1/invitation.proto",
    "scylla/oauth/v1/oauth.proto",
    "scylla/user/v1/user.proto",
    "scylla/organization/v1/organization.proto",
    "scylla/project/v1/project.proto",
    "scylla/authz/v1/permission.proto",
    "scylla/authz/v1/policy.proto",
    "scylla/authz/v1/grant.proto",
    "scylla/authz/v1/role.proto",
    "scylla/pipeline/v1/pipeline.proto",
    "scylla/secret/v1/secret.proto",
    "scylla/job/v1/job.proto",
    "scylla/app/v1/app.proto",
    "scylla/app/v1/app_auth.proto",
    "scylla/agent/v1/agent_stream.proto",
    "scylla/agent/v1/agent_admin.proto",
    "scylla/trigger/v1/trigger.proto",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let proto_root = manifest_dir.join("proto");

    let protos: Vec<PathBuf> = PROTOS.iter().map(|f| proto_root.join(f)).collect();

    for proto in &protos {
        println!("cargo:rerun-if-changed={}", proto.display());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("scylla_descriptor.bin"))
        // A single include root. Two roots would make the same file addressable
        // by two paths, which protoc treats as two different files.
        .compile_protos(&protos, &[proto_root])?;

    Ok(())
}
