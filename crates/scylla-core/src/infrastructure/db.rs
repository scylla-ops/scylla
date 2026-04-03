use crate::domain::entities::{
    JobId, OrganizationId, PipelineId, ProjectId, SessionId, UserId, UserOrganizationId,
    UserProjectId,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub namespace: String,
    pub database: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "memory".to_string(),
            namespace: "scylla".to_string(),
            database: "core".to_string(),
        }
    }
}

pub async fn init_db(config: &DatabaseConfig) -> Result<Surreal<Any>> {
    let db = surrealdb::engine::any::connect(&config.url)
        .await
        .with_context(|| format!("Failed to connect to database at {}", config.url))?;

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
        if let Some(err) = errors.get(&i) {
            tracing::error!(table, %err, "Failed to define table");
        } else {
            tracing::debug!(table, "Table defined successfully");
        }
    }

    if !errors.is_empty() {
        anyhow::bail!("Database schema init failed for {} table(s)", errors.len());
    }

    Ok(db)
}
