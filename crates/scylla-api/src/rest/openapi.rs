use utoipa::OpenApi;

use super::response::*;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Scylla API",
        version = "1.0.0",
        description = "Scylla CI/CD pipeline orchestration API",
    ),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Auth", description = "Authentication"),
        (name = "Users", description = "User management"),
        (name = "Organizations", description = "Organization management"),
        (name = "Projects", description = "Project management"),
        (name = "Pipelines", description = "Pipeline management"),
        (name = "Jobs", description = "Job management"),
        (name = "Permissions", description = "Permission and RBAC management"),
    ),
    paths(
        // Health
        super::routes::health::health,
        super::routes::health::ready,
        // Auth
        super::routes::auth::login,
        super::routes::auth::validate_token,
        super::routes::auth::revoke_token,
        // Users
        super::routes::users::create,
        super::routes::users::get_one,
        super::routes::users::update,
        super::routes::users::delete_one,
        super::routes::users::list,
        super::routes::users::list_user_organizations,
        super::routes::users::list_user_projects,
        // Organizations
        super::routes::organizations::create,
        super::routes::organizations::get_one,
        super::routes::organizations::update,
        super::routes::organizations::delete_one,
        super::routes::organizations::list,
        super::routes::organizations::toggle_active,
        super::routes::organizations::list_users,
        super::routes::organizations::add_user,
        super::routes::organizations::remove_user,
        super::routes::organizations::list_pipelines,
        super::routes::organizations::list_jobs,
        // Projects
        super::routes::projects::create,
        super::routes::projects::get_one,
        super::routes::projects::update,
        super::routes::projects::delete_one,
        super::routes::projects::list,
        super::routes::projects::toggle_active,
        super::routes::projects::list_users,
        super::routes::projects::add_user,
        super::routes::projects::remove_user,
        super::routes::projects::list_pipelines,
        super::routes::projects::list_jobs,
        // Pipelines
        super::routes::pipelines::create,
        super::routes::pipelines::get_one,
        super::routes::pipelines::update,
        super::routes::pipelines::delete_one,
        super::routes::pipelines::list,
        super::routes::pipelines::list_jobs,
        // Jobs
        super::routes::jobs::get_one,
        super::routes::jobs::delete_one,
        super::routes::jobs::list,
        // Permissions
        super::routes::permissions::add_policy,
        super::routes::permissions::remove_policy,
        super::routes::permissions::list_policies,
        super::routes::permissions::add_grouping_policy,
        super::routes::permissions::remove_grouping_policy,
        super::routes::permissions::list_grouping_policies,
    ),
    components(schemas(
        ErrorBody,
        HealthResponse,
        // Auth
        LoginRequest, LoginResponse, TokenRequest, ValidateTokenResponse,
        // Users
        CreateUserRequest, UpdateUserRequest, UserResponse, ListUsersResponse,
        // Organizations
        CreateOrganizationRequest, UpdateOrganizationRequest,
        OrganizationResponse, ListOrganizationsResponse,
        AddUserRequest, MemberResponse, ListMembersResponse,
        ListUserOrganizationsResponse,
        // Projects
        CreateProjectRequest, UpdateProjectRequest,
        ProjectResponse, ListProjectsResponse,
        // Pipelines
        CreatePipelineRequest, UpdatePipelineRequest, PipelineNodeRequest,
        PipelineResponse, PipelineNodeResponse,
        PipelineSummaryResponse, ListPipelinesResponse,
        // Jobs
        JobResponse, JobNodeResponse, ListJobsResponse,
        // Permissions
        PolicyRequest, PolicyResponse, ListPoliciesResponse,
        GroupingPolicyRequest, GroupingPolicyResponse, ListGroupingPoliciesResponse,
        BoolResponse,
        // Pagination
        PaginationQuery, PaginationMeta,
    )),
    security(
        ("bearer_auth" = [])
    ),
    modifiers(&SecurityAddon),
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            utoipa::openapi::security::SecurityScheme::Http(utoipa::openapi::security::Http::new(
                utoipa::openapi::security::HttpAuthScheme::Bearer,
            )),
        );
    }
}
