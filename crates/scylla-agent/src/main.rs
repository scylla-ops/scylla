use anyhow::Context;
use clap::Parser;
use scylla_agent::{Agent, AgentConfig};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = AgentConfig::parse();
    ensure_workspace_root(&config.workspace_root)?;
    info!(
        control_plane_url = %config.control_plane_url,
        app_id = %config.app_id,
        workspace_root = %config.workspace_root.display(),
        "starting scylla-agent"
    );

    let agent = Agent::new(config);

    tokio::select! {
        result = agent.run() => result.map_err(anyhow::Error::from),
        () = shutdown_signal() => {
            info!("shutdown signal received");
            Ok(())
        }
    }
}

/// Fail fast on an unusable workspace root: create it if missing and probe it
/// with a real write. Without this, a bad `--workspace-root` (e.g. the default
/// `/var/lib/scylla/workspaces` absent on a dev machine) is only discovered
/// when the first job fails.
fn ensure_workspace_root(root: &std::path::Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(root).with_context(|| {
        format!(
            "workspace root {} cannot be created — pass a writable directory via \
             --workspace-root or SCYLLA_WORKSPACE_ROOT",
            root.display()
        )
    })?;
    let probe = root.join(".scylla-write-probe");
    std::fs::write(&probe, b"probe").with_context(|| {
        format!(
            "workspace root {} is not writable — pass a writable directory via \
             --workspace-root or SCYLLA_WORKSPACE_ROOT",
            root.display()
        )
    })?;
    let _ = std::fs::remove_file(&probe);
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        // INVARIANT: SIGTERM handler installation cannot fail at startup on supported platforms.
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
}
