#[cfg(feature = "users")]
use crate::domain::entities::UserId;
#[cfg(feature = "auth")]
use crate::domain::entities::SessionId;
#[cfg(feature = "organizations")]
use crate::domain::entities::{OrganizationId, UserOrganizationId};
#[cfg(feature = "projects")]
use crate::domain::entities::{ProjectId, UserProjectId};
#[cfg(feature = "pipelines")]
use crate::domain::entities::PipelineId;
#[cfg(feature = "jobs")]
use crate::domain::entities::{JobId, JobLogId};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::opt::auth::Root;

/// Type alias for the shared SurrealDB client.
pub type Db = Surreal<Any>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub namespace: String,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "memory".to_string(),
            namespace: "scylla".to_string(),
            database: "core".to_string(),
            username: None,
            password: None,
        }
    }
}

pub async fn init_db(config: &DatabaseConfig) -> Result<Surreal<Any>> {
    let db = surrealdb::engine::any::connect(&config.url)
        .await
        .with_context(|| format!("Failed to connect to database at {}", config.url))?;

    if let (Some(username), Some(password)) = (&config.username, &config.password) {
        db.signin(Root {
            username: username.clone(),
            password: password.clone(),
        })
        .await?;
    }

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
    #[cfg(feature = "jobs")]
    tables.push(JobLogId::table_name());

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

/// Explicitly invalidate the SurrealDB session so the websocket close frame
/// is sent before the tokio runtime shuts down. Without this, Windows Ctrl+C
/// kills the runtime before the `Drop` impl can clean up, leaving SurrealDB
/// with zombie sessions that block the next startup.
pub async fn close_db(db: &Db) {
    if let Err(e) = db.invalidate().await {
        tracing::warn!(error = %e, "failed to invalidate SurrealDB session");
    }
    tracing::debug!("SurrealDB session invalidated");
}
