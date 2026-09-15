use scylla_auth::audit::{AuditEntry, AuditLog};
use sqlx::PgPool;
use tokio::sync::mpsc;
use tracing::warn;

/// Unbounded queue: completeness over bounded memory; switch to bounded + batched inserts if the writer lags.
pub struct PgAuditLog {
    tx: mpsc::UnboundedSender<AuditEntry>,
}

impl PgAuditLog {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(writer_loop(pool, rx));
        Self { tx }
    }
}

impl AuditLog for PgAuditLog {
    fn record(&self, entry: AuditEntry) {
        if self.tx.send(entry).is_err() {
            warn!("audit writer task is gone; dropping audit entry");
        }
    }
}

async fn writer_loop(pool: PgPool, mut rx: mpsc::UnboundedReceiver<AuditEntry>) {
    while let Some(entry) = rx.recv().await {
        if let Err(e) = insert(&pool, &entry).await {
            warn!(error = %e, action = entry.action, "failed to persist audit entry");
        }
    }
}

async fn insert(pool: &PgPool, entry: &AuditEntry) -> Result<(), sqlx::Error> {
    let id = scylla_domain::domain::ids::new_id();
    sqlx::query(
        "INSERT INTO audit_log \
         (id, occurred_at, principal_kind, principal_id, action, resource_kind, resource_id, decision, policies, reason) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(id)
    .bind(entry.occurred_at)
    .bind(entry.principal_kind)
    .bind(entry.principal_id.as_deref())
    .bind(entry.action)
    .bind(entry.resource_kind)
    .bind(entry.resource_id.as_deref())
    .bind(entry.decision.as_str())
    .bind(&entry.policies)
    .bind(entry.reason.as_deref())
    .execute(pool)
    .await?;
    Ok(())
}
