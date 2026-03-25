use scylla_core::domain::entities::*;
use scylla_core::domain::value_objects::PaginationMetadata;
use scylla_core::domain::value_objects::job::{JobStatus, NodeState};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// --- Pagination ---

#[derive(Deserialize, ToSchema)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Serialize, ToSchema)]
pub struct PaginationMeta {
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_previous: bool,
}

impl From<&PaginationMetadata> for PaginationMeta {
    fn from(m: &PaginationMetadata) -> Self {
        Self {
            total_count: m.total_count(),
            page: m.page(),
            page_size: m.page_size(),
            total_pages: m.total_pages(),
            has_next: m.has_next(),
            has_previous: m.has_previous(),
        }
    }
}

// --- Auth ---

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
}

#[derive(Deserialize, ToSchema)]
pub struct TokenRequest {
    pub token: String,
}

#[derive(Serialize, ToSchema)]
pub struct ValidateTokenResponse {
    pub is_valid: bool,
}

// --- Users ---

#[derive(Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub user_id: String,
    pub username: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&User> for UserResponse {
    fn from(u: &User) -> Self {
        Self {
            user_id: u.id().to_string(),
            username: u.username().to_string(),
            is_active: u.is_active(),
            created_at: u.created_at().to_rfc3339(),
            updated_at: u.updated_at().to_rfc3339(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ListUsersResponse {
    pub users: Vec<UserResponse>,
    pub pagination: PaginationMeta,
}

// --- Organizations ---

#[derive(Deserialize, ToSchema)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateOrganizationRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct OrganizationResponse {
    pub organization_id: String,
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&Organization> for OrganizationResponse {
    fn from(o: &Organization) -> Self {
        Self {
            organization_id: o.id().to_string(),
            name: o.name().to_string(),
            description: o.description().map(|d| d.to_string()).unwrap_or_default(),
            is_active: o.is_active(),
            created_at: o.created_at().to_rfc3339(),
            updated_at: o.updated_at().to_rfc3339(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ListOrganizationsResponse {
    pub organizations: Vec<OrganizationResponse>,
    pub pagination: PaginationMeta,
}

#[derive(Deserialize, ToSchema)]
pub struct AddUserRequest {
    pub user_id: String,
}

#[derive(Serialize, ToSchema)]
pub struct MemberResponse {
    pub user_id: String,
    pub username: String,
}

#[derive(Serialize, ToSchema)]
pub struct ListMembersResponse {
    pub members: Vec<MemberResponse>,
    pub pagination: PaginationMeta,
}

#[derive(Serialize, ToSchema)]
pub struct ListUserOrganizationsResponse {
    pub organizations: Vec<OrganizationResponse>,
    pub pagination: PaginationMeta,
}

// --- Projects ---

#[derive(Deserialize, ToSchema)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub organization_id: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ProjectResponse {
    pub project_id: String,
    pub name: String,
    pub description: String,
    pub organization_id: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&Project> for ProjectResponse {
    fn from(p: &Project) -> Self {
        Self {
            project_id: p.id().to_string(),
            name: p.name().to_string(),
            description: p.description().map(|d| d.to_string()).unwrap_or_default(),
            organization_id: p.organization_id().to_string(),
            is_active: p.is_active(),
            created_at: p.created_at().to_rfc3339(),
            updated_at: p.updated_at().to_rfc3339(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ListProjectsResponse {
    pub projects: Vec<ProjectResponse>,
    pub pagination: PaginationMeta,
}

// --- Pipelines ---

#[derive(Deserialize, ToSchema)]
pub struct CreatePipelineRequest {
    pub project_id: String,
    pub name: String,
    pub nodes: Vec<PipelineNodeRequest>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdatePipelineRequest {
    pub name: Option<String>,
    pub nodes: Option<Vec<PipelineNodeRequest>>,
}

#[derive(Deserialize, ToSchema)]
pub struct PipelineNodeRequest {
    pub node_id: String,
    pub deps: Vec<String>,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct PipelineResponse {
    pub pipeline_id: String,
    pub project_id: String,
    pub name: String,
    pub nodes: Vec<PipelineNodeResponse>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, ToSchema)]
pub struct PipelineNodeResponse {
    pub node_id: String,
    pub deps: Vec<String>,
    pub command: String,
    pub args: Vec<String>,
}

impl From<&Pipeline> for PipelineResponse {
    fn from(p: &Pipeline) -> Self {
        Self {
            pipeline_id: p.id().to_string(),
            project_id: p.project_id().to_string(),
            name: p.name().to_string(),
            nodes: p.nodes().iter().map(PipelineNodeResponse::from).collect(),
            created_at: p.created_at().to_rfc3339(),
            updated_at: p.updated_at().to_rfc3339(),
        }
    }
}

impl From<&PipelineNode> for PipelineNodeResponse {
    fn from(n: &PipelineNode) -> Self {
        Self {
            node_id: n.id().to_string(),
            deps: n.deps().iter().map(|d| d.to_string()).collect(),
            command: n.command().to_string(),
            args: n.args().to_vec(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct PipelineSummaryResponse {
    pub pipeline_id: String,
    pub project_id: String,
    pub name: String,
    pub node_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&Pipeline> for PipelineSummaryResponse {
    fn from(p: &Pipeline) -> Self {
        Self {
            pipeline_id: p.id().to_string(),
            project_id: p.project_id().to_string(),
            name: p.name().to_string(),
            node_count: p.nodes().len(),
            created_at: p.created_at().to_rfc3339(),
            updated_at: p.updated_at().to_rfc3339(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ListPipelinesResponse {
    pub pipelines: Vec<PipelineSummaryResponse>,
    pub pagination: PaginationMeta,
}

// --- Jobs ---

#[derive(Serialize, ToSchema)]
pub struct JobResponse {
    pub job_id: String,
    pub pipeline_id: String,
    pub status: String,
    pub node_executions: Vec<JobNodeResponse>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, ToSchema)]
pub struct JobNodeResponse {
    pub node_id: String,
    pub state: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

impl From<&Job> for JobResponse {
    fn from(j: &Job) -> Self {
        Self {
            job_id: j.id().to_string(),
            pipeline_id: j.pipeline_id().to_string(),
            status: format_job_status(j.status()),
            node_executions: j
                .node_executions()
                .iter()
                .map(JobNodeResponse::from)
                .collect(),
            created_at: j.created_at().to_rfc3339(),
            updated_at: j.updated_at().to_rfc3339(),
        }
    }
}

impl From<&JobNode> for JobNodeResponse {
    fn from(n: &JobNode) -> Self {
        Self {
            node_id: n.node_id().to_string(),
            state: format_node_state(n.state()),
            started_at: n.started_at().map(|t| t.to_rfc3339()),
            finished_at: n.finished_at().map(|t| t.to_rfc3339()),
        }
    }
}

fn format_job_status(status: JobStatus) -> String {
    match status {
        JobStatus::Pending => "pending",
        JobStatus::Running => "running",
        JobStatus::Completed => "completed",
        JobStatus::Failed => "failed",
        JobStatus::Cancelled => "cancelled",
        JobStatus::Orphaned => "orphaned",
    }
    .to_string()
}

fn format_node_state(state: NodeState) -> String {
    match state {
        NodeState::Pending => "pending",
        NodeState::Running => "running",
        NodeState::Completed => "completed",
        NodeState::Failed => "failed",
        NodeState::Cancelled => "cancelled",
    }
    .to_string()
}

#[derive(Serialize, ToSchema)]
pub struct ListJobsResponse {
    pub jobs: Vec<JobResponse>,
    pub pagination: PaginationMeta,
}

// --- Permissions ---

#[derive(Deserialize, ToSchema)]
pub struct PolicyRequest {
    pub subject: String,
    pub scope: String,
    pub scope_id: Option<String>,
    pub resource: String,
    pub resource_id: Option<String>,
    pub act: String,
}

#[derive(Serialize, ToSchema)]
pub struct PolicyResponse {
    pub subject: String,
    pub scope: String,
    pub resource: String,
    pub act: String,
}

#[derive(Serialize, ToSchema)]
pub struct ListPoliciesResponse {
    pub policies: Vec<PolicyResponse>,
}

#[derive(Deserialize, ToSchema)]
pub struct GroupingPolicyRequest {
    pub subject: String,
    pub role: String,
    pub scope: String,
    pub scope_id: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct GroupingPolicyResponse {
    pub subject: String,
    pub role: String,
    pub scope: String,
}

#[derive(Serialize, ToSchema)]
pub struct ListGroupingPoliciesResponse {
    pub policies: Vec<GroupingPolicyResponse>,
}

#[derive(Serialize, ToSchema)]
pub struct BoolResponse {
    pub success: bool,
}

// --- Error ---

#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: String,
}

// --- Health ---

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
}
