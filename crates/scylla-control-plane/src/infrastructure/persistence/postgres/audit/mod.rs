use crate::application::audit::{AuditEntry, AuditLog};
use sqlx::PgPool;
use tokio::sync::mpsc;
use tracing::warn;

/// Persists audit entries to the `audit_log` table **out-of-band**: `record`
/// only enqueues onto an unbounded channel, and a background task drains it and
/// inserts. The authorization hot path therefore never waits on the database.
///
/// Trade-off: an unbounded queue favours completeness (no dropped audit) over
/// bounded memory; if the writer can't keep up under sustained load, switch to a
/// bounded channel + batched inserts.
pub struct PgAuditLog {
    tx: mpsc::UnboundedSender<AuditEntry>,
}

impl PgAuditLog {
    /// Spawns the background writer. Must be called within a Tokio runtime.
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
            // Never fail the caller for an audit write; surface and continue.
            warn!(error = %e, action = entry.action, "failed to persist audit entry");
        }
    }
}

async fn insert(pool: &PgPool, entry: &AuditEntry) -> Result<(), sqlx::Error> {
    let id = scylla_core::domain::ids::new_id();
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
