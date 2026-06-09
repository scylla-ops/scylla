use crate::grpc::convert::{ts, wrap};
use scylla_core::domain::entities::Project;
use scylla_protocol::services::project::ProjectResponse;

pub fn project_to_proto(project: &Project) -> ProjectResponse {
    ProjectResponse {
        project_id: wrap(project.id().to_string()),
        name: project.name().to_string(),
        description: project
            .description()
            .map(|d| d.as_str().to_string())
            .unwrap_or_default(),
        organization_id: wrap(project.organization_id().to_string()),
        is_active: project.is_active(),
        created_at: ts(project.created_at()),
        updated_at: ts(project.updated_at()),
    }
}
