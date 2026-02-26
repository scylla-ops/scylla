use async_trait::async_trait;
use casbin::{Adapter, Filter, Model, Result as CasbinResult};
use surrealdb::{Surreal, engine::any::Any};
use surrealdb_types::SurrealValue;

pub const TABLE: &str = "casbin_rule";

#[derive(Debug, Clone)]
struct RuleValues {
    v0: Option<String>,
    v1: Option<String>,
    v2: Option<String>,
    v3: Option<String>,
    v4: Option<String>,
    v5: Option<String>,
}

impl RuleValues {
    fn from_slice(rule: &[String]) -> Self {
        let get = |i: usize| rule.get(i).cloned();
        Self {
            v0: get(0),
            v1: get(1),
            v2: get(2),
            v3: get(3),
            v4: get(4),
            v5: get(5),
        }
    }

    fn to_vec(&self) -> Vec<String> {
        [&self.v0, &self.v1, &self.v2, &self.v3, &self.v4, &self.v5]
            .iter()
            .filter_map(|v| v.as_deref().map(str::to_owned))
            .collect()
    }
}

#[derive(Debug, Clone)]
struct ExactRuleParams {
    sec: String,
    ptype: String,
    values: RuleValues,
}

impl ExactRuleParams {
    fn new(sec: &str, ptype: &str, rule: &[String]) -> Self {
        Self {
            sec: sec.to_owned(),
            ptype: ptype.to_owned(),
            values: RuleValues::from_slice(rule),
        }
    }
}

#[derive(Debug, Clone)]
struct FilteredRuleParams {
    sec: String,
    ptype: String,
    field_index: usize,
    field_values: Vec<String>,
}

impl FilteredRuleParams {
    fn new(sec: &str, ptype: &str, field_index: usize, field_values: Vec<String>) -> Self {
        Self {
            sec: sec.to_owned(),
            ptype: ptype.to_owned(),
            field_index,
            field_values,
        }
    }
}

// ─── CasbinRule ──────────────────────────────────────────────────────────────
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
    fn new(sec: &str, ptype: &str, rule: &[String]) -> Self {
        let v = RuleValues::from_slice(rule);
        Self {
            sec: sec.to_owned(),
            ptype: ptype.to_owned(),
            v0: v.v0,
            v1: v.v1,
            v2: v.v2,
            v3: v.v3,
            v4: v.v4,
            v5: v.v5,
        }
    }

    fn to_rule(&self) -> Vec<String> {
        RuleValues {
            v0: self.v0.clone(),
            v1: self.v1.clone(),
            v2: self.v2.clone(),
            v3: self.v3.clone(),
            v4: self.v4.clone(),
            v5: self.v5.clone(),
        }
        .to_vec()
    }
}

// ─── load helper ─────────────────────────────────────────────────────────────
fn load_policy_line(m: &mut dyn Model, rule: &CasbinRule) {
    let values = rule.to_rule();
    if values.is_empty() {
        return;
    }
    if let Some(sec_map) = m.get_mut_model().get_mut(&rule.sec) {
        if let Some(assertion) = sec_map.get_mut(&rule.ptype) {
            assertion.get_mut_policy().insert(values);
        }
    }
}

// ─── Adapter ─────────────────────────────────────────────────────────────────

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

    pub async fn create_table(&self) {
        self.db
            .query("DEFINE TABLE IF NOT EXISTS $table SCHEMALESS;")
            .bind(("table", TABLE))
            .await
            .ok();
    }
}

#[async_trait]
impl Adapter for SurrealAdapter {
    async fn load_policy(&mut self, m: &mut dyn Model) -> CasbinResult<()> {
        for rule in self.get_all_rules().await? {
            load_policy_line(m, &rule);
        }
        self.is_filtered = false;
        Ok(())
    }

    async fn load_filtered_policy<'a>(
        &mut self,
        m: &mut dyn Model,
        f: Filter<'a>,
    ) -> CasbinResult<()> {
        for rule in self.get_all_rules().await? {
            let values = rule.to_rule();

            let filter = match rule.sec.as_str() {
                "p" => &f.p,
                "g" => &f.g,
                _ => continue,
            };

            let matches = filter
                .iter()
                .enumerate()
                .all(|(i, fv)| fv.is_empty() || values.get(i).map(|v| v == fv).unwrap_or(false));

            if matches {
                load_policy_line(m, &rule);
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
                        all_rules.push(CasbinRule::new(sec, ptype, policy));
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
        let _: Vec<CasbinRule> = self.db.delete(TABLE).await.map_err(io_err)?;
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
        let params = ExactRuleParams::new(sec, ptype, &rule);
        if self.rule_exists(&params).await? {
            return Ok(false);
        }
        let entry = CasbinRule::new(sec, ptype, &rule);
        let _: Option<CasbinRule> = self.db.create(TABLE).content(entry).await.map_err(io_err)?;
        Ok(true)
    }

    async fn add_policies(
        &mut self,
        sec: &str,
        ptype: &str,
        rules: Vec<Vec<String>>,
    ) -> CasbinResult<bool> {
        for rule in &rules {
            let params = ExactRuleParams::new(sec, ptype, rule);
            if self.rule_exists(&params).await? {
                return Ok(false);
            }
        }
        let entries: Vec<CasbinRule> = rules
            .iter()
            .map(|r| CasbinRule::new(sec, ptype, r))
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
        let params = ExactRuleParams::new(sec, ptype, &rule);
        self.delete_exact(&params).await
    }

    async fn remove_policies(
        &mut self,
        sec: &str,
        ptype: &str,
        rules: Vec<Vec<String>>,
    ) -> CasbinResult<bool> {
        let mut removed_any = false;
        for rule in rules {
            let params = ExactRuleParams::new(sec, ptype, &rule);
            if self.delete_exact(&params).await? {
                removed_any = true;
            }
        }
        Ok(removed_any)
    }

    async fn remove_filtered_policy(
        &mut self,
        sec: &str,
        ptype: &str,
        field_index: usize,
        field_values: Vec<String>,
    ) -> CasbinResult<bool> {
        let params = FilteredRuleParams::new(sec, ptype, field_index, field_values);
        self.delete_filtered(&params).await
    }
}

// ─── Private helpers ─────────────────────────────────────────────────────────

impl SurrealAdapter {
    async fn insert_entries(&self, entries: Vec<CasbinRule>) -> CasbinResult<bool> {
        let _: Vec<CasbinRule> = self
            .db
            .insert(TABLE)
            .content(entries)
            .await
            .map_err(io_err)?;
        Ok(true)
    }

    async fn get_all_rules(&self) -> CasbinResult<Vec<CasbinRule>> {
        self.db.select(TABLE).await.map_err(io_err)
    }

    async fn rule_exists(&self, params: &ExactRuleParams) -> CasbinResult<bool> {
        let found: Vec<CasbinRule> = self
            .db
            .query(
                "SELECT * FROM type::table($table)
                 WHERE sec = $sec AND ptype = $ptype
                   AND v0 = $v0 AND v1 = $v1 AND v2 = $v2
                   AND v3 = $v3 AND v4 = $v4 AND v5 = $v5",
            )
            .bind(("table", TABLE))
            .bind(("sec", params.sec.clone()))
            .bind(("ptype", params.ptype.clone()))
            .bind(("v0", params.values.v0.clone()))
            .bind(("v1", params.values.v1.clone()))
            .bind(("v2", params.values.v2.clone()))
            .bind(("v3", params.values.v3.clone()))
            .bind(("v4", params.values.v4.clone()))
            .bind(("v5", params.values.v5.clone()))
            .await
            .map_err(io_err)?
            .take(0)
            .map_err(io_err)?;

        Ok(!found.is_empty())
    }

    async fn delete_exact(&self, params: &ExactRuleParams) -> CasbinResult<bool> {
        let deleted: Vec<CasbinRule> = self
            .db
            .query(
                "DELETE type::table($table)
                 WHERE sec = $sec AND ptype = $ptype
                   AND v0 = $v0 AND v1 = $v1 AND v2 = $v2
                   AND v3 = $v3 AND v4 = $v4 AND v5 = $v5
                 RETURN BEFORE",
            )
            .bind(("table", TABLE))
            .bind(("sec", params.sec.clone()))
            .bind(("ptype", params.ptype.clone()))
            .bind(("v0", params.values.v0.clone()))
            .bind(("v1", params.values.v1.clone()))
            .bind(("v2", params.values.v2.clone()))
            .bind(("v3", params.values.v3.clone()))
            .bind(("v4", params.values.v4.clone()))
            .bind(("v5", params.values.v5.clone()))
            .await
            .map_err(io_err)?
            .take(0)
            .map_err(io_err)?;

        Ok(!deleted.is_empty())
    }

    async fn delete_filtered(&self, params: &FilteredRuleParams) -> CasbinResult<bool> {
        let col_conditions: String = params
            .field_values
            .iter()
            .enumerate()
            .filter(|(_, v)| !v.is_empty())
            .map(|(offset, _)| {
                let col = params.field_index + offset;
                let bind = format!("fv{}", offset);
                format!("v{} = ${}", col, bind)
            })
            .collect::<Vec<_>>()
            .join(" AND ");

        let where_clause = if col_conditions.is_empty() {
            "sec = $sec AND ptype = $ptype".to_owned()
        } else {
            format!("sec = $sec AND ptype = $ptype AND {}", col_conditions)
        };

        let query_str = format!(
            "DELETE type::table($table) WHERE {} RETURN BEFORE",
            where_clause
        );

        let mut q = self
            .db
            .query(query_str)
            .bind(("table", TABLE))
            .bind(("sec", params.sec.clone()))
            .bind(("ptype", params.ptype.clone()));

        for (offset, v) in params
            .field_values
            .iter()
            .enumerate()
            .filter(|(_, v)| !v.is_empty())
        {
            q = q.bind((format!("fv{}", offset), v.clone()));
        }

        let deleted: Vec<CasbinRule> = q.await.map_err(io_err)?.take(0).map_err(io_err)?;
        Ok(!deleted.is_empty())
    }
}

// ─── Error helper ─────────────────────────────────────────────────────────────
fn io_err(e: impl std::fmt::Display) -> casbin::Error {
    casbin::Error::IoError(std::io::Error::new(
        std::io::ErrorKind::Other,
        e.to_string(),
    ))
}
