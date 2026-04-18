use scylla_core::domain::entities::Organization;
use scylla_protocol::services::organization::OrganizationResponse;

pub fn organization_to_proto(org: &Organization) -> OrganizationResponse {
    OrganizationResponse {
        organization_id: org.id().to_string(),
        name: org.name().to_string(),
        description: org
            .description()
            .map(|d| d.as_str().to_string())
            .unwrap_or_default(),
        is_active: org.is_active(),
        created_at: org.created_at().to_rfc3339(),
        updated_at: org.updated_at().to_rfc3339(),
    }
}
