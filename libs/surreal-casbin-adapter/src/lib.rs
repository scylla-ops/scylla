use async_trait::async_trait;
use casbin::{Adapter, Filter, Model, Result as CasbinResult};
use surrealdb::{
    engine::any::Any,
    Surreal,
};
use surrealdb_types::SurrealValue;
// ─── Constantes ────────────────────────────────────────────────────────────────

pub const TABLE: &str = "casbin_rule";

// ─── Structure d'une règle stockée en SurrealDB ─────────────────────────────
#[derive(Debug, Clone, SurrealValue)]
struct CasbinRule {
    sec: String,
    ptype: String,
    v0: Option<String>,
    v1: Option<String>,
    v2: Option<String>,
    v3: Option<String>,
    v4: Option<String>,
    v5: Option<String>,
}

impl CasbinRule {
    fn from_rule(sec: &str, ptype: &str, rule: &[String]) -> Self {
        let get = |i: usize| rule.get(i).cloned();
        Self {
            sec: sec.to_owned(),
            ptype: ptype.to_owned(),
            v0: get(0),
            v1: get(1),
            v2: get(2),
            v3: get(3),
            v4: get(4),
            v5: get(5),
        }
    }

    fn to_rule(&self) -> Vec<String> {
        [&self.v0, &self.v1, &self.v2, &self.v3, &self.v4, &self.v5]
            .iter()
            .filter_map(|v| v.as_deref().map(str::to_owned))
            .collect()
    }
}

// ─── Adapter ────────────────────────────────────────────────────────────────

/// Adapter casbin utilisant SurrealDB 3.0 comme backend.
///
/// # Exemple
///
/// ```rust,no_run
/// use casbin::{CoreApi, DefaultModel, Enforcer};
/// use surrealdb::{Surreal, engine::any::connect, opt::auth::Root};
/// use casbin_surreal_adapter::SurrealAdapter;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let db = connect("ws://localhost:8000").await?;
///     db.signin(Root { username: "root", password: "root" }).await?;
///     db.use_ns("my_ns").use_db("my_db").await?;
///
///     let adapter = SurrealAdapter::new(db);
///     let model = DefaultModel::from_file("rbac_model.conf").await?;
///     let enforcer = Enforcer::new(model, adapter).await?;
///     Ok(())
/// }
/// ```
pub struct SurrealAdapter {
    db: Surreal<Any>,
    is_filtered: bool,
}

impl SurrealAdapter {
    pub fn new(db: Surreal<Any>) -> Self {
        Self {
            db,
            is_filtered: false,
        }
    }
}

// ─── Implémentation du trait Adapter ─────────────────────────────────────────

#[async_trait]
impl Adapter for SurrealAdapter {
    async fn load_policy(&mut self, m: &mut dyn Model) -> CasbinResult<()> {
        let rules = self.get_all_rules().await?;

        for rule in rules {
            load_rule_into_model(m, &rule);
        }

        self.is_filtered = false;
        Ok(())
    }

    async fn load_filtered_policy<'a>(
        &mut self,
        m: &mut dyn Model,
        f: Filter<'a>,
    ) -> CasbinResult<()> {
        let rules = self.get_all_rules().await?;

        for rule in &rules {
            let values = rule.to_rule();
            let filter = if rule.sec == "p" { &f.p } else { &f.g };

            let matches = filter.iter().enumerate().all(|(i, fv)| {
                fv.is_empty() || values.get(i).map(|v| v == fv).unwrap_or(false)
            });

            if matches {
                load_rule_into_model(m, rule);
            }
        }

        self.is_filtered = true;
        Ok(())
    }

    async fn save_policy(&mut self, m: &mut dyn Model) -> CasbinResult<()> {
        self.clear_policy().await?;

        let mut all_rules: Vec<CasbinRule> = Vec::new();

        for sec in ["p", "g"] {
            if let Some(sec_map) = m.get_model().get(sec) {
                for (ptype, assertion) in sec_map {
                    for policy in assertion.get_policy() {
                        all_rules.push(CasbinRule::from_rule(sec, ptype, policy));
                    }
                }
            }
        }

        if !all_rules.is_empty() {
            self.insert_entries(all_rules).await?;
        }

        Ok(())
    }

    async fn clear_policy(&mut self) -> CasbinResult<()> {
        let _: Vec<CasbinRule> = self
            .db
            .delete(TABLE)
            .await
            .map_err(|e| casbin::Error::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))?;
        Ok(())
    }

    fn is_filtered(&self) -> bool {
        self.is_filtered
    }

    async fn add_policy(
        &mut self,
        sec: &str,
        ptype: &str,
        rule: Vec<String>,
    ) -> CasbinResult<bool> {
        if self.rule_exists(sec, ptype, &rule).await? {
            return Ok(false);
        }

        let entry = CasbinRule::from_rule(sec, ptype, &rule);
        let _: Option<CasbinRule> = self
            .db
            .create(TABLE)
            .content(entry)
            .await
            .map_err(|e| casbin::Error::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))?;

        Ok(true)
    }

    async fn add_policies(
        &mut self,
        sec: &str,
        ptype: &str,
        rules: Vec<Vec<String>>,
    ) -> CasbinResult<bool> {
        for rule in &rules {
            if self.rule_exists(sec, ptype, rule).await? {
                return Ok(false);
            }
        }

        let entries: Vec<CasbinRule> = rules
            .iter()
            .map(|r| CasbinRule::from_rule(sec, ptype, r))
            .collect();

        self.insert_entries(entries).await?;

        Ok(true)
    }

    async fn remove_policy(
        &mut self,
        sec: &str,
        ptype: &str,
        rule: Vec<String>,
    ) -> CasbinResult<bool> {
        let query = build_delete_query(sec, ptype, &rule, None, &[]);
        let mut result = self
            .db
            .query(query)
            .await
            .map_err(|e| casbin::Error::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))?;

        let deleted: Vec<CasbinRule> = result
            .take(0)
            .map_err(|e| casbin::Error::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))?;

        Ok(!deleted.is_empty())
    }

    /// Supprime plusieurs règles en batch.
    async fn remove_policies(
        &mut self,
        sec: &str,
        ptype: &str,
        rules: Vec<Vec<String>>,
    ) -> CasbinResult<bool> {
        let mut removed_any = false;
        for rule in rules {
            if self.remove_policy(sec, ptype, rule).await? {
                removed_any = true;
            }
        }
        Ok(removed_any)
    }

    /// Supprime les règles dont les champs à partir de `field_index`
    /// correspondent aux `field_values` donnés.
    async fn remove_filtered_policy(
        &mut self,
        sec: &str,
        ptype: &str,
        field_index: usize,
        field_values: Vec<String>,
    ) -> CasbinResult<bool> {
        let query = build_delete_query(sec, ptype, &[], Some(field_index), &field_values);
        let mut result = self
            .db
            .query(query)
            .await
            .map_err(|e| casbin::Error::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))?;

        let deleted: Vec<CasbinRule> = result
            .take(0)
            .map_err(|e| casbin::Error::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))?;

        Ok(!deleted.is_empty())
    }
}

// ─── Helpers privés ──────────────────────────────────────────────────────────

impl SurrealAdapter {
    async fn insert_entries(&self, entries: Vec<CasbinRule>) -> CasbinResult<bool> {
        let _: Vec<CasbinRule> = self
            .db
            .insert(TABLE)
            .content(entries)
            .await
            .map_err(|e| casbin::Error::IoError(
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
            ))?;

        Ok(true)
    }

    async fn get_all_rules(&self) -> CasbinResult<Vec<CasbinRule>> {
        let rules: Vec<CasbinRule> = self
            .db
            .select(TABLE)
            .await
            .map_err(|e| casbin::Error::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))?;
        Ok(rules)
    }

    async fn rule_exists(
        &self,
        sec: &str,
        ptype: &str,
        rule: &[String],
    ) -> CasbinResult<bool> {
        let query = build_select_query(sec, ptype, rule);
        let mut result = self
            .db
            .query(query)
            .await
            .map_err(|e| casbin::Error::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))?;

        let found: Vec<CasbinRule> = result
            .take(0)
            .map_err(|e| casbin::Error::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))?;

        Ok(!found.is_empty())
    }
}

/// Construit une requête SELECT SurrealQL pour trouver une règle exacte.
fn build_select_query(sec: &str, ptype: &str, rule: &[String]) -> String {
    let mut conditions = format!(
        "sec = '{}' AND ptype = '{}'",
        escape(sec),
        escape(ptype)
    );

    for (i, v) in rule.iter().enumerate() {
        conditions.push_str(&format!(" AND v{} = '{}'", i, escape(v)));
    }

    // S'assure que les colonnes non fournies sont NULL
    for i in rule.len()..6 {
        conditions.push_str(&format!(" AND v{} = NONE", i));
    }

    format!("SELECT * FROM {} WHERE {}", TABLE, conditions)
}

/// Construit une requête DELETE SurrealQL.
///
/// - Si `rule` est non-vide : suppression d'une règle exacte.
/// - Si `field_index` + `field_values` sont fournis : suppression filtrée.
fn build_delete_query(
    sec: &str,
    ptype: &str,
    rule: &[String],
    field_index: Option<usize>,
    field_values: &[String],
) -> String {
    let mut conditions = format!(
        "sec = '{}' AND ptype = '{}'",
        escape(sec),
        escape(ptype)
    );

    if !rule.is_empty() {
        // Suppression exacte
        for (i, v) in rule.iter().enumerate() {
            conditions.push_str(&format!(" AND v{} = '{}'", i, escape(v)));
        }
        for i in rule.len()..6 {
            conditions.push_str(&format!(" AND v{} = NONE", i));
        }
    } else if let Some(idx) = field_index {
        // Suppression filtrée à partir de field_index
        for (offset, v) in field_values.iter().enumerate() {
            if !v.is_empty() {
                conditions.push_str(&format!(
                    " AND v{} = '{}'",
                    idx + offset,
                    escape(v)
                ));
            }
        }
    }

    format!("DELETE {} WHERE {} RETURN BEFORE", TABLE, conditions)
}

fn escape(s: &str) -> String {
    s.replace('\'', "\\'")
}

fn load_rule_into_model(m: &mut dyn Model, rule: &CasbinRule) {
    let values = rule.to_rule();
    if values.is_empty() {
        return;
    }

    let line = values.join(", ");
    m.add_policy(&rule.sec, &rule.ptype, values);
    let _ = line;
}