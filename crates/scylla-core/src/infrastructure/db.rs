use crate::domain::entities::{UserId, SessionId, OrganizationId, UserOrganizationId, ProjectId, UserProjectId, PipelineId, JobId};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub database: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "ws://127.0.0.1:8000".to_string(),
            username: "root".to_string(),
            password: "secret".to_string(),
            namespace: "scylla".to_string(),
            database: "core".to_string(),
        }
    }
}

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

    let mut tables: Vec<&str> = Vec::new();

    #[cfg(feature = "users")]
    tables.push(UserId::table_name());
    #[cfg(feature = "auth")]
    tables.push(SessionId::table_name());
    #[cfg(feature = "organizations")]
    {
        tables.push(OrganizationId::table_name());
        tables.push(UserOrganizationId::table_name());
    }
    #[cfg(feature = "projects")]
    {
        tables.push(ProjectId::table_name());
        tables.push(UserProjectId::table_name());
    }
    #[cfg(feature = "pipelines")]
    tables.push(PipelineId::table_name());
    #[cfg(feature = "jobs")]
    tables.push(JobId::table_name());

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
        if let Some(err) = errors.get(&i) { tracing::error!(table, %err, "Failed to define table") } else { tracing::debug!(table, "Table defined successfully") }
    }

    if !errors.is_empty() {
        anyhow::bail!("Database schema init failed for {} table(s)", errors.len());
    }

    Ok(db)
}
