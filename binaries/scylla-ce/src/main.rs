//! The Community Edition binary: the public server with the default
//! extensions. Every step below is edition-independent except the
//! `Extensions` literal, which is the whole difference between editions.

use scylla_core::application::UnlimitedQuota;
use scylla_core::config::ControlPlaneConfig;
use scylla_extension::Extensions;
use scylla_server::cli::{self, Cli};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse_as(
        "scylla-ce",
        env!("CARGO_PKG_VERSION"),
        "Scylla Community Edition",
    );
    if cli.print_example_config {
        ControlPlaneConfig::print_example();
        return Ok(());
    }
    cli::init_tracing(&["scylla_ce"]);
    let config = cli.load_config()?;
    let db = scylla_db::init_db(&config.database).await?;
    let extensions = Extensions {
        quota: Arc::new(UnlimitedQuota),
    };
    scylla_server::serve(config, db, extensions).await
}
