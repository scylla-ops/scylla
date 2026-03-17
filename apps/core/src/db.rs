use crate::config::DatabaseConfig;
use anyhow::{Context, Result};
use domain::entities::{
    JobId, OrganizationId, PipelineId, ProjectId, SessionId, UserId, UserOrganizationId,
    UserProjectId,
};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

pub async fn init_db(config: &DatabaseConfig) -> Result<Surreal<Any>> {
    let db: Surreal<Any> = Surreal::init();

    db.connect(&config.url)
        .await
        .with_context(|| format!("Failed to connect to database at {}", config.url))?;

    db.signin(surrealdb::opt::auth::Root {
        username: config.username.clone(),
        password: config.password.clone(),
    })
    .await
    .context("Failed to authenticate with database")?;

    db.use_ns(&config.namespace)
        .use_db(&config.database)
        .await
        .context("Failed to select namespace/database")?;

    let tables = [
        UserId::table_name(),
        SessionId::table_name(),
        OrganizationId::table_name(),
        ProjectId::table_name(),
        UserOrganizationId::table_name(),
        UserProjectId::table_name(),
        PipelineId::table_name(),
        JobId::table_name(),
    ];

    let ddl = tables
        .iter()
        .map(|t| format!("DEFINE TABLE IF NOT EXISTS {t} SCHEMALESS;"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut response = db
        .query(ddl)
        .await
        .context("Failed to initialize database tables")?;

    let errors = response.take_errors();

    for (i, table) in tables.iter().enumerate() {
        match errors.get(&i) {
            Some(err) => tracing::error!(table, %err, "Failed to define table"),
            None => tracing::debug!(table, "Table defined successfully"),
        }
    }

    if !errors.is_empty() {
        anyhow::bail!("Database schema init failed for {} table(s)", errors.len());
    }

    Ok(db)
}
