use super::models::BlacklistRecord;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::repositories::blacklist_repository::BlacklistRepository;
use crate::domain::value_objects::BlacklistId;
use async_trait::async_trait;
use derive_more::Constructor;
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

#[derive(Constructor)]
pub struct SurrealBlacklistRepository {
    db: Arc<Surreal<Any>>,
}

#[async_trait]
impl BlacklistRepository for SurrealBlacklistRepository {
    async fn is_blacklisted(&self, item: &str) -> DomainResult<bool> {
        let mut result = self
            .db
            .query("SELECT * FROM type::table($table) WHERE item = $item")
            .bind(("table", BlacklistId::table_name()))
            .bind(("item", item.to_owned()))
            .await
            .map_err(|e| {
                DomainError::infrastructure(format!("Database error checking blacklist: {}", e))
            })?;

        let records: Vec<BlacklistRecord> = result.take(0).map_err(|e| {
            DomainError::infrastructure(format!("Query error checking blacklist: {}", e))
        })?;

        Ok(!records.is_empty())
    }

    async fn add_to_blacklist(&self, item: String) -> DomainResult<()> {
        self.db
            .query("CREATE type::table($table) SET item = $item, created_at = time::now()")
            .bind(("table", BlacklistId::table_name()))
            .bind(("item", item))
            .await
            .map_err(|e| {
                DomainError::infrastructure(format!("Database error adding to blacklist: {}", e))
            })?;
        Ok(())
    }

    async fn remove_from_blacklist(&self, item: &str) -> DomainResult<()> {
        self.db
            .query("DELETE type::table($table) WHERE item = $item")
            .bind(("table", BlacklistId::table_name()))
            .bind(("item", item.to_owned()))
            .await
            .map_err(|e| {
                DomainError::infrastructure(format!(
                    "Database error removing from blacklist: {}",
                    e
                ))
            })?;
        Ok(())
    }
}
