use crate::postgres::{PgAuthzEntityProvider, PgGrantRepository, PgRoleRepository};
use scylla_auth::audit::NoopAuditLog;
use scylla_auth::cedar::CedarPermissionService;
use sqlx::PgPool;
use std::sync::Arc;

pub async fn cedar(pool: &PgPool) -> Arc<CedarPermissionService<PgAuthzEntityProvider>> {
    Arc::new(
        CedarPermissionService::new(
            Arc::new(PgAuthzEntityProvider::new(pool.clone())),
            Arc::new(PgRoleRepository::new(pool.clone())),
            Arc::new(PgGrantRepository::new(pool.clone())),
            Arc::new(NoopAuditLog),
        )
        .await
        .expect("cedar"),
    )
}
