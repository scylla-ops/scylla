//! # SurrealDB Adapter for Casbin
//!
//! This crate provides a SurrealDB adapter for [Casbin](https://casbin.org/), an authorization
//! library that supports access control models like ACL, RBAC, ABAC for Rust projects.
//!
//! ## Features
//!
//! - Async/await support
//! - All Casbin adapter operations (load, save, add, remove policies)
//! - Filtered policy loading
//! - Generic over SurrealDB connection types
//!
//! ## Usage
//!
//! ```rust,no_run
//! use surreal_casbin_adapter::SurrealAdapter;
//! use casbin::prelude::*;
//! use surrealdb::Surreal;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize your SurrealDB client
//! let db = Surreal::new::<surrealdb::engine::remote::ws::Ws>("127.0.0.1:8000").await?;
//! db.signin(surrealdb::opt::auth::Root {
//!     username: "root",
//!     password: "root",
//! }).await?;
//! db.use_ns("test").use_db("test").await?;
//!
//! // Create the adapter with a custom table name
//! let adapter = SurrealAdapter::new(Arc::new(db), "casbin_rules");
//!
//! // Create a Casbin enforcer
//! let mut enforcer = Enforcer::new("model.conf", adapter).await?;
//!
//! // Use the enforcer
//! enforcer.add_policy(vec!["alice".to_string(), "data1".to_string(), "read".to_string()]).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Database Schema
//!
//! The adapter expects a table with the following structure (SurrealQL):
//!
//! ```sql
//! DEFINE TABLE casbin_rules SCHEMAFULL;
//! DEFINE FIELD ptype ON TABLE casbin_rules TYPE string;
//! DEFINE FIELD v0 ON TABLE casbin_rules TYPE option<string>;
//! DEFINE FIELD v1 ON TABLE casbin_rules TYPE option<string>;
//! DEFINE FIELD v2 ON TABLE casbin_rules TYPE option<string>;
//! DEFINE FIELD v3 ON TABLE casbin_rules TYPE option<string>;
//! DEFINE FIELD v4 ON TABLE casbin_rules TYPE option<string>;
//! DEFINE FIELD v5 ON TABLE casbin_rules TYPE option<string>;
//! DEFINE INDEX casbin_ptype_idx ON TABLE casbin_rules COLUMNS ptype;
//! ```

use async_trait::async_trait;
use casbin::{Adapter, Filter, Model, Result as CasbinResult};
use serde::{Deserialize, Serialize};
use surrealdb::Connection;

/// Represents a row in the casbin_rules table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasbinRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<surrealdb::RecordId>,
    pub ptype: String,
    pub v0: Option<String>,
    pub v1: Option<String>,
    pub v2: Option<String>,
    pub v3: Option<String>,
    pub v4: Option<String>,
    pub v5: Option<String>,
}

impl CasbinRule {
    /// Creates a CasbinRule from a policy type and rule
    fn from_policy(ptype: &str, rule: &[String]) -> Self {
        let mut r = CasbinRule {
            id: None,
            ptype: ptype.to_owned(),
            v0: None,
            v1: None,
            v2: None,
            v3: None,
            v4: None,
            v5: None,
        };

        if !rule.is_empty() {
            r.v0 = Some(rule[0].clone());
        }
        if rule.len() > 1 {
            r.v1 = Some(rule[1].clone());
        }
        if rule.len() > 2 {
            r.v2 = Some(rule[2].clone());
        }
        if rule.len() > 3 {
            r.v3 = Some(rule[3].clone());
        }
        if rule.len() > 4 {
            r.v4 = Some(rule[4].clone());
        }
        if rule.len() > 5 {
            r.v5 = Some(rule[5].clone());
        }

        r
    }

    /// Converts a CasbinRule to a policy vector
    fn to_policy(&self) -> Vec<String> {
        let mut policy = Vec::new();

        if let Some(ref v) = self.v0 {
            policy.push(v.clone());
        }
        if let Some(ref v) = self.v1 {
            policy.push(v.clone());
        }
        if let Some(ref v) = self.v2 {
            policy.push(v.clone());
        }
        if let Some(ref v) = self.v3 {
            policy.push(v.clone());
        }
        if let Some(ref v) = self.v4 {
            policy.push(v.clone());
        }
        if let Some(ref v) = self.v5 {
            policy.push(v.clone());
        }

        policy
    }
}

/// SurrealDB adapter for Casbin
///
/// This adapter implements the Casbin Adapter trait to store and retrieve
/// authorization policies from a SurrealDB database.
///
/// # Type Parameters
///
/// * `C` - The SurrealDB connection type (e.g., `surrealdb::engine::remote::ws::Client`)
pub struct SurrealAdapter<C>
where
    C: Connection,
{
    db: std::sync::Arc<surrealdb::Surreal<C>>,
    table_name: String,
}

impl<C> SurrealAdapter<C>
where
    C: Connection,
{
    /// Creates a new SurrealAdapter
    ///
    /// # Arguments
    ///
    /// * `db` - A SurrealDB client instance
    /// * `table_name` - The name of the table to store Casbin rules (default: "casbin_rules")
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use surreal_casbin_adapter::SurrealAdapter;
    /// use surrealdb::Surreal;
    /// use std::sync::Arc;
    ///
    /// let db = Surreal::new::<surrealdb::engine::remote::ws::Ws>("127.0.0.1:8000").await?;
    /// let adapter = SurrealAdapter::new(Arc::new(db), "casbin_rules");
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(db: impl Into<std::sync::Arc<surrealdb::Surreal<C>>>, table_name: impl Into<String>) -> Self {
        Self {
            db: db.into(),
            table_name: table_name.into(),
        }
    }

    /// Loads a policy line into the model
    fn load_policy_line(&self, rule: &CasbinRule, model: &mut dyn Model) {
        let sec = &rule.ptype[0..1];
        let ptype = &rule.ptype;
        let policy = rule.to_policy();

        if let Some(ast_map) = model.get_mut_model().get_mut(sec) {
            if let Some(ast) = ast_map.get_mut(ptype) {
                ast.policy.insert(policy);
            }
        }
    }

    /// Internal method to load policies with optional filtering
    async fn load_filtered_policy_internal<'a>(
        &self,
        model: &mut dyn Model,
        filter: Option<Filter<'a>>,
    ) -> CasbinResult<()> {
        let rules: Vec<CasbinRule> = if let Some(filter) = filter {
            // build filtered query based on filter
            let mut query_str = format!("SELECT * FROM {}", self.table_name);
            let mut conditions = Vec::new();

            if !filter.p.is_empty() {
                conditions.push(format!("ptype = '{}'", filter.p[0]));
            }
            if !filter.g.is_empty() {
                conditions.push(format!("ptype = '{}'", filter.g[0]));
            }

            if !conditions.is_empty() {
                query_str.push_str(" WHERE ");
                query_str.push_str(&conditions.join(" OR "));
            }

            let mut result = self.db.query(query_str).await.map_err(|e| {
                casbin::error::AdapterError(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to load filtered policies: {}", e),
                )))
            })?;

            result.take(0).map_err(|e| {
                casbin::error::AdapterError(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to parse policies: {}", e),
                )))
            })?
        } else {
            // load all rules
            self.db
                .select(&self.table_name)
                .await
                .map_err(|e| {
                    casbin::error::AdapterError(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to load policies: {}", e),
                    )))
                })?
        };

        for rule in rules {
            self.load_policy_line(&rule, model);
        }

        Ok(())
    }

    /// Internal method to save all policies
    async fn save_policy_internal(&self, model: &mut dyn Model) -> CasbinResult<()> {
        // first, delete all existing rules
        self.db
            .query(format!("DELETE FROM {}", self.table_name))
            .await
            .map_err(|e| {
                casbin::error::AdapterError(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to clear policies: {}", e),
                )))
            })?;

        let mut rules = Vec::new();

        // collect all policies from model
        if let Some(ast_map) = model.get_model().get("p") {
            for (ptype, ast) in ast_map {
                for policy in &ast.policy {
                    let rule = CasbinRule::from_policy(ptype, policy);
                    rules.push(rule);
                }
            }
        }

        // collect all role definitions from model
        if let Some(ast_map) = model.get_model().get("g") {
            for (ptype, ast) in ast_map {
                for policy in &ast.policy {
                    let rule = CasbinRule::from_policy(ptype, policy);
                    rules.push(rule);
                }
            }
        }

        // insert all rules
        if !rules.is_empty() {
            for rule in rules {
                let _: Option<CasbinRule> = self
                    .db
                    .create(&self.table_name)
                    .content(rule)
                    .await
                    .map_err(|e| {
                        casbin::error::AdapterError(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to save policy: {}", e),
                        )))
                    })?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl<C> Adapter for SurrealAdapter<C>
where
    C: Connection,
{
    async fn load_policy(&mut self, model: &mut dyn Model) -> CasbinResult<()> {
        self.load_filtered_policy_internal(model, None).await
    }

    async fn save_policy(&mut self, model: &mut dyn Model) -> CasbinResult<()> {
        self.save_policy_internal(model).await
    }

    async fn clear_policy(&mut self) -> CasbinResult<()> {
        self.db
            .query(format!("DELETE FROM {}", self.table_name))
            .await
            .map_err(|e| {
                casbin::error::AdapterError(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to clear policies: {}", e),
                )))
            })?;
        Ok(())
    }

    async fn load_filtered_policy<'a>(
        &mut self,
        model: &mut dyn Model,
        filter: Filter<'a>,
    ) -> CasbinResult<()> {
        self.load_filtered_policy_internal(model, Some(filter))
            .await
    }

    fn is_filtered(&self) -> bool {
        false
    }

    async fn add_policy(&mut self, _sec: &str, ptype: &str, rule: Vec<String>) -> CasbinResult<bool> {
        let casbin_rule = CasbinRule::from_policy(ptype, &rule);

        let _: Option<CasbinRule> = self
            .db
            .create(&self.table_name)
            .content(casbin_rule)
            .await
            .map_err(|e| {
                casbin::error::AdapterError(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to add policy: {}", e),
                )))
            })?;

        Ok(true)
    }

    async fn add_policies(
        &mut self,
        _sec: &str,
        ptype: &str,
        rules: Vec<Vec<String>>,
    ) -> CasbinResult<bool> {
        for rule in rules {
            let casbin_rule = CasbinRule::from_policy(ptype, &rule);
            let _: Option<CasbinRule> = self
                .db
                .create(&self.table_name)
                .content(casbin_rule)
                .await
                .map_err(|e| {
                    casbin::error::AdapterError(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to add policy: {}", e),
                    )))
                })?;
        }

        Ok(true)
    }

    async fn remove_policy(&mut self, _sec: &str, ptype: &str, rule: Vec<String>) -> CasbinResult<bool> {
        // build a query to find and delete the matching rule
        let mut query = format!(
            "DELETE FROM {} WHERE ptype = $ptype",
            self.table_name
        );

        for (i, _) in rule.iter().enumerate() {
            query.push_str(&format!(" AND v{} = $v{}", i, i));
        }

        let mut q = self.db.query(query);
        q = q.bind(("ptype", ptype.to_string()));

        for (i, value) in rule.iter().enumerate() {
            q = q.bind((format!("v{}", i), value.clone()));
        }

        q.await.map_err(|e| {
            casbin::error::AdapterError(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to remove policy: {}", e),
            )))
        })?;

        Ok(true)
    }

    async fn remove_policies(
        &mut self,
        sec: &str,
        ptype: &str,
        rules: Vec<Vec<String>>,
    ) -> CasbinResult<bool> {
        for rule in rules {
            self.remove_policy(sec, ptype, rule).await?;
        }
        Ok(true)
    }

    async fn remove_filtered_policy(
        &mut self,
        _sec: &str,
        ptype: &str,
        field_index: usize,
        field_values: Vec<String>,
    ) -> CasbinResult<bool> {
        let mut query = format!(
            "DELETE FROM {} WHERE ptype = $ptype",
            self.table_name
        );

        let mut bind_params = vec![];
        for (i, value) in field_values.iter().enumerate() {
            if !value.is_empty() {
                let param_name = format!("v{}", field_index + i);
                query.push_str(&format!(" AND {} = ${}", param_name, param_name));
                bind_params.push((param_name, value.clone()));
            }
        }

        let mut q = self.db.query(query);
        q = q.bind(("ptype", ptype.to_string()));

        for (param_name, value) in bind_params {
            q = q.bind((param_name, value));
        }

        q.await.map_err(|e| {
            casbin::error::AdapterError(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to remove filtered policy: {}", e),
            )))
        })?;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_casbin_rule_from_policy() {
        let rule = CasbinRule::from_policy("p", &["alice".to_string(), "data1".to_string(), "read".to_string()]);
        assert_eq!(rule.ptype, "p");
        assert_eq!(rule.v0, Some("alice".to_string()));
        assert_eq!(rule.v1, Some("data1".to_string()));
        assert_eq!(rule.v2, Some("read".to_string()));
        assert_eq!(rule.v3, None);
    }

    #[test]
    fn test_casbin_rule_to_policy() {
        let rule = CasbinRule {
            id: None,
            ptype: "p".to_string(),
            v0: Some("alice".to_string()),
            v1: Some("data1".to_string()),
            v2: Some("read".to_string()),
            v3: None,
            v4: None,
            v5: None,
        };
        let policy = rule.to_policy();
        assert_eq!(policy, vec!["alice", "data1", "read"]);
    }
}
