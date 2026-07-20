use crate::grpc::convert::{ts, wrap};
use scylla_core::domain::entities::Organization;
use scylla_protocol::organization::v1::Organization as ProtoOrganization;

pub fn organization_to_proto(org: &Organization) -> ProtoOrganization {
    ProtoOrganization {
        organization_id: wrap(org.id().to_string()),
        name: org.name().to_string(),
        description: org
            .description()
            .map(|d| d.as_str().to_string())
            .unwrap_or_default(),
        is_active: org.is_active(),
        created_at: ts(org.created_at()),
        updated_at: ts(org.updated_at()),
    }
}
