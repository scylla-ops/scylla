use crate::grpc::convert::{ts, wrap};
use scylla_core::domain::secret::Secret as DomainSecret;
use scylla_protocol::secret::v1::Secret;

/// Domain secret → proto. Metadata only; the value is never included.
pub fn secret_to_proto(secret: &DomainSecret) -> Secret {
    Secret {
        secret_id: wrap(secret.id().to_string()),
        project_id: wrap(secret.project_id().to_string()),
        name: secret.name().as_str().to_string(),
        description: secret.description().to_string(),
        created_at: ts(secret.created_at()),
        updated_at: ts(secret.updated_at()),
    }
}
