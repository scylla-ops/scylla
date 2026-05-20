pub mod resource_ref;

pub use resource_ref::ResourceRef;

use crate::domain::entities::{AgentId, JobId, OrganizationId, PipelineId, ProjectId, UserId};

/// Authorization intent: a named operation plus the concrete resource it acts
/// on. This is the single vocabulary the application layer uses to ask "is the
/// caller allowed to do X?". It carries no Cedar types — the infra adapter maps
/// `action()` to a Cedar `Action::"…"` and `resource()` to a typed entity.
///
/// One variant per operation (fine-grained actions) so the Cedar schema can pin
/// `appliesTo` per action and policies stay readable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Permission {
    // ── user ───────────────────────────────────────────────────────────
    CreateUser,
    ReadUser(UserId),
    UpdateUser(UserId),
    DeleteUser(UserId),
    ListUsers,

    // ── organization ───────────────────────────────────────────────────
    CreateOrganization,
    ReadOrganization(OrganizationId),
    UpdateOrganization(OrganizationId),
    DeleteOrganization(OrganizationId),
    ListOrganizations,
    ListOrganizationMembers(OrganizationId),
    AddOrganizationMember(OrganizationId),
    RemoveOrganizationMember(OrganizationId),
    ListUserOrganizations(UserId),

    // ── project ────────────────────────────────────────────────────────
    CreateProject(OrganizationId),
    ReadProject(ProjectId),
    UpdateProject(ProjectId),
    DeleteProject(ProjectId),
    ListProjects,
    ListProjectsByOrganization(OrganizationId),
    ListProjectMembers(ProjectId),
    AddProjectMember(ProjectId),
    RemoveProjectMember(ProjectId),
    ListUserProjects(UserId),

    // ── pipeline ───────────────────────────────────────────────────────
    CreatePipeline(ProjectId),
    ReadPipeline(PipelineId),
    UpdatePipeline(PipelineId),
    DeletePipeline(PipelineId),
    RunPipeline(PipelineId),
    ListPipelines,
    ListPipelinesByProject(ProjectId),
    ListPipelinesByOrganization(OrganizationId),

    // ── job ────────────────────────────────────────────────────────────
    CreateJob,
    ReadJob(JobId),
    WriteJob(JobId),
    DeleteJob(JobId),
    ListJobs,
    ListJobsByPipeline(PipelineId),
    ListJobsByProject(ProjectId),
    ListJobsByOrganization(OrganizationId),
    ReadJobLogs(JobId),
    WriteJobLogs(JobId),

    // ── agent ──────────────────────────────────────────────────────────
    ReadAgent(AgentId),
    WriteAgent(AgentId),
    DeleteAgent(AgentId),
    ListAgents,

    // ── grants (admin) ─────────────────────────────────────────────────
    ManageGrants,
}

impl Permission {
    /// Canonical action identifier — becomes the Cedar `Action::"<id>"` eid.
    #[must_use]
    pub fn action(&self) -> &'static str {
        match self {
            Self::CreateUser => "createUser",
            Self::ReadUser(_) => "readUser",
            Self::UpdateUser(_) => "updateUser",
            Self::DeleteUser(_) => "deleteUser",
            Self::ListUsers => "listUsers",

            Self::CreateOrganization => "createOrganization",
            Self::ReadOrganization(_) => "readOrganization",
            Self::UpdateOrganization(_) => "updateOrganization",
            Self::DeleteOrganization(_) => "deleteOrganization",
            Self::ListOrganizations => "listOrganizations",
            Self::ListOrganizationMembers(_) => "listOrganizationMembers",
            Self::AddOrganizationMember(_) => "addOrganizationMember",
            Self::RemoveOrganizationMember(_) => "removeOrganizationMember",
            Self::ListUserOrganizations(_) => "listUserOrganizations",

            Self::CreateProject(_) => "createProject",
            Self::ReadProject(_) => "readProject",
            Self::UpdateProject(_) => "updateProject",
            Self::DeleteProject(_) => "deleteProject",
            Self::ListProjects => "listProjects",
            Self::ListProjectsByOrganization(_) => "listProjectsByOrganization",
            Self::ListProjectMembers(_) => "listProjectMembers",
            Self::AddProjectMember(_) => "addProjectMember",
            Self::RemoveProjectMember(_) => "removeProjectMember",
            Self::ListUserProjects(_) => "listUserProjects",

            Self::CreatePipeline(_) => "createPipeline",
            Self::ReadPipeline(_) => "readPipeline",
            Self::UpdatePipeline(_) => "updatePipeline",
            Self::DeletePipeline(_) => "deletePipeline",
            Self::RunPipeline(_) => "runPipeline",
            Self::ListPipelines => "listPipelines",
            Self::ListPipelinesByProject(_) => "listPipelinesByProject",
            Self::ListPipelinesByOrganization(_) => "listPipelinesByOrganization",

            Self::CreateJob => "createJob",
            Self::ReadJob(_) => "readJob",
            Self::WriteJob(_) => "writeJob",
            Self::DeleteJob(_) => "deleteJob",
            Self::ListJobs => "listJobs",
            Self::ListJobsByPipeline(_) => "listJobsByPipeline",
            Self::ListJobsByProject(_) => "listJobsByProject",
            Self::ListJobsByOrganization(_) => "listJobsByOrganization",
            Self::ReadJobLogs(_) => "readJobLogs",
            Self::WriteJobLogs(_) => "writeJobLogs",

            Self::ReadAgent(_) => "readAgent",
            Self::WriteAgent(_) => "writeAgent",
            Self::DeleteAgent(_) => "deleteAgent",
            Self::ListAgents => "listAgents",

            Self::ManageGrants => "manageGrants",
        }
    }

    /// The concrete resource the action targets. Create / list-all / global
    /// operations target the `System` singleton; everything else targets the
    /// specific entity (or the parent scope for scoped lists/creates).
    #[must_use]
    pub fn resource(&self) -> ResourceRef {
        match self {
            // System-scoped (admin / service in practice)
            Self::CreateUser
            | Self::ListUsers
            | Self::CreateOrganization
            | Self::ListOrganizations
            | Self::ListProjects
            | Self::ListPipelines
            | Self::CreateJob
            | Self::ListJobs
            | Self::ListAgents
            | Self::ManageGrants => ResourceRef::System,

            // User-targeted
            Self::ReadUser(id)
            | Self::UpdateUser(id)
            | Self::DeleteUser(id)
            | Self::ListUserOrganizations(id)
            | Self::ListUserProjects(id) => ResourceRef::User(id.clone()),

            // Organization-targeted
            Self::ReadOrganization(id)
            | Self::UpdateOrganization(id)
            | Self::DeleteOrganization(id)
            | Self::ListOrganizationMembers(id)
            | Self::AddOrganizationMember(id)
            | Self::RemoveOrganizationMember(id)
            | Self::CreateProject(id)
            | Self::ListProjectsByOrganization(id)
            | Self::ListPipelinesByOrganization(id)
            | Self::ListJobsByOrganization(id) => ResourceRef::Organization(id.clone()),

            // Project-targeted
            Self::ReadProject(id)
            | Self::UpdateProject(id)
            | Self::DeleteProject(id)
            | Self::ListProjectMembers(id)
            | Self::AddProjectMember(id)
            | Self::RemoveProjectMember(id)
            | Self::CreatePipeline(id)
            | Self::ListPipelinesByProject(id)
            | Self::ListJobsByProject(id) => ResourceRef::Project(id.clone()),

            // Pipeline-targeted
            Self::ReadPipeline(id)
            | Self::UpdatePipeline(id)
            | Self::DeletePipeline(id)
            | Self::RunPipeline(id)
            | Self::ListJobsByPipeline(id) => ResourceRef::Pipeline(id.clone()),

            // Job-targeted
            Self::ReadJob(id)
            | Self::WriteJob(id)
            | Self::DeleteJob(id)
            | Self::ReadJobLogs(id)
            | Self::WriteJobLogs(id) => ResourceRef::Job(id.clone()),

            // Agent-targeted
            Self::ReadAgent(id) | Self::WriteAgent(id) | Self::DeleteAgent(id) => {
                ResourceRef::Agent(id.clone())
            }
        }
    }
}
