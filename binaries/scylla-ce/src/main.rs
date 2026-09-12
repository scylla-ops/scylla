//! The Community Edition binary: the public server with its defaults. Every
//! step below is edition-independent; an edition differs by what it adds to
//! the [`Server`] between `new` and `serve`, and this one adds nothing.

use scylla_core::config::ControlPlaneConfig;
use scylla_server::Server;
use scylla_server::cli::{self, Cli};

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
    Server::new(config, db).serve().await
}
