use scylla_core::domain::entities::Project;
use scylla_protocol::services::project::ProjectResponse;

pub fn project_to_proto(project: &Project) -> ProjectResponse {
    ProjectResponse {
        project_id: project.id().to_string(),
        name: project.name().to_string(),
        description: project
            .description()
            .map(|d| d.as_str().to_string())
            .unwrap_or_default(),
        organization_id: project.organization_id().to_string(),
        is_active: project.is_active(),
        created_at: project.created_at().to_rfc3339(),
        updated_at: project.updated_at().to_rfc3339(),
    }
}
