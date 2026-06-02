pub mod resource_ref;

pub use resource_ref::ResourceRef;

use crate::domain::entities::{AppId, JobId, OrganizationId, PipelineId, ProjectId, UserId};

/// Authorization intent: a named operation plus the concrete resource it acts
/// on. This is the single vocabulary the application layer uses to ask "is the
/// caller allowed to do X?". It carries no Cedar types — the infra adapter maps
/// `key()` to a Cedar `Action::"…"` and `resource()` to a typed entity.
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
    /// Manage this organization's invitations (create / revoke / list pending).
    /// Distinct from member listing so a plain org member can't enumerate
    /// outstanding invites (and their invitee emails) — only org-admins can.
    ManageInvitations(OrganizationId),
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
    /// A agent App executing the jobs of a pipeline (distinct from a user
    /// triggering a run via `RunPipeline`).
    ExecuteJob(PipelineId),
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
    /// A agent App reporting a job's status / appending its logs while it runs.
    WriteJobStatus(JobId),
    WriteJobLog(JobId),

    // ── app (machine principals) ───────────────────────────────────────
    CreateApp(OrganizationId),
    ReadApp(AppId),
    DeleteApp(AppId),
    ListAppsByOrganization(OrganizationId),

    // ── agent (specialized apps that run jobs) ─────────────────────────
    CreateAgent(OrganizationId),
    ListAgents(OrganizationId),
    ReadAgent(AppId),
    ReadAgentStats(AppId),
    DeleteAgent(AppId),

    // ── grants / policies / roles ──────────────────────────────────────
    /// System-scoped grant management (admin / service): manage any grant.
    ManageGrants,
    /// Manage grants whose scope is this organization (org-admins). Cedar
    /// hierarchy bounds it to the org and the projects beneath it, so it cannot
    /// be used to touch grants in another org (anti-escalation).
    ManageOrgGrants(OrganizationId),
    /// Manage grants whose scope is this project (project-admins).
    ManageProjectGrants(ProjectId),
    ManagePolicies,
    /// Create / edit / delete roles (the dynamic role catalog). System-scoped.
    ManageRoles,
}

impl Permission {
    /// Canonical permission key — becomes the Cedar `Action::"<id>"` eid.
    #[must_use]
    pub fn key(&self) -> &'static str {
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
            Self::ManageInvitations(_) => "manageInvitations",
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
            Self::ExecuteJob(_) => "executeJob",
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
            Self::WriteJobStatus(_) => "writeJobStatus",
            Self::WriteJobLog(_) => "writeJobLog",

            Self::CreateApp(_) => "createApp",
            Self::ReadApp(_) => "readApp",
            Self::DeleteApp(_) => "deleteApp",
            Self::ListAppsByOrganization(_) => "listAppsByOrganization",

            Self::CreateAgent(_) => "createAgent",
            Self::ListAgents(_) => "listAgents",
            Self::ReadAgent(_) => "readAgent",
            Self::ReadAgentStats(_) => "readAgentStats",
            Self::DeleteAgent(_) => "deleteAgent",

            // Distinct action ids per scope so the Cedar schema pins `appliesTo`
            // (System / Organization / Project) per action. A single shared
            // "manageGrants" would let one over-broad permit on it authorize all
            // three scopes (scope load-bearing only via the resource arm).
            Self::ManageGrants => "manageGrants",
            Self::ManageOrgGrants(_) => "manageOrgGrants",
            Self::ManageProjectGrants(_) => "manageProjectGrants",
            Self::ManagePolicies => "managePolicies",
            Self::ManageRoles => "manageRoles",
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
            | Self::ManageGrants
            | Self::ManagePolicies
            | Self::ManageRoles => ResourceRef::System,

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
            | Self::ManageInvitations(id)
            | Self::CreateProject(id)
            | Self::ListProjectsByOrganization(id)
            | Self::ListPipelinesByOrganization(id)
            | Self::ListJobsByOrganization(id)
            | Self::CreateApp(id)
            | Self::ListAppsByOrganization(id)
            | Self::CreateAgent(id)
            | Self::ListAgents(id)
            | Self::ManageOrgGrants(id) => ResourceRef::Organization(id.clone()),

            // Project-targeted
            Self::ReadProject(id)
            | Self::UpdateProject(id)
            | Self::DeleteProject(id)
            | Self::ListProjectMembers(id)
            | Self::AddProjectMember(id)
            | Self::RemoveProjectMember(id)
            | Self::CreatePipeline(id)
            | Self::ListPipelinesByProject(id)
            | Self::ListJobsByProject(id)
            | Self::ManageProjectGrants(id) => ResourceRef::Project(id.clone()),

            // Pipeline-targeted
            Self::ReadPipeline(id)
            | Self::UpdatePipeline(id)
            | Self::DeletePipeline(id)
            | Self::RunPipeline(id)
            | Self::ExecuteJob(id)
            | Self::ListJobsByPipeline(id) => ResourceRef::Pipeline(id.clone()),

            // Job-targeted
            Self::ReadJob(id)
            | Self::WriteJob(id)
            | Self::DeleteJob(id)
            | Self::ReadJobLogs(id)
            | Self::WriteJobLogs(id)
            | Self::WriteJobStatus(id)
            | Self::WriteJobLog(id) => ResourceRef::Job(id.clone()),

            Self::ReadApp(id) | Self::DeleteApp(id) => ResourceRef::App(id.clone()),

            Self::ReadAgent(id) | Self::ReadAgentStats(id) | Self::DeleteAgent(id) => {
                ResourceRef::App(id.clone())
            }
        }
    }
}

/// Every resource type a policy may target — the `Scylla::<Type>` entities, by
/// their lowercase tag. Mirrors [`ResourceRef`].
pub const RESOURCE_TYPES: &[&str] = &[
    "system",
    "user",
    "organization",
    "project",
    "pipeline",
    "job",
    "app",
];

/// The full authorization vocabulary: every permission key paired with the
/// resource type it targets. Drives `ListAuthzVocabulary` so a policy author sees what a
/// `permit`/`forbid` may reference instead of guessing names. One row per
/// [`Permission`] variant — keep in sync (the test below asserts every row's
/// resource type is one of [`RESOURCE_TYPES`] and ids are unique).
pub const PERMISSION_CATALOG: &[(&str, &str)] = &[
    // user
    ("createUser", "system"),
    ("readUser", "user"),
    ("updateUser", "user"),
    ("deleteUser", "user"),
    ("listUsers", "system"),
    // organization
    ("createOrganization", "system"),
    ("readOrganization", "organization"),
    ("updateOrganization", "organization"),
    ("deleteOrganization", "organization"),
    ("listOrganizations", "system"),
    ("listOrganizationMembers", "organization"),
    ("addOrganizationMember", "organization"),
    ("removeOrganizationMember", "organization"),
    ("manageInvitations", "organization"),
    ("listUserOrganizations", "user"),
    // project
    ("createProject", "organization"),
    ("readProject", "project"),
    ("updateProject", "project"),
    ("deleteProject", "project"),
    ("listProjects", "system"),
    ("listProjectsByOrganization", "organization"),
    ("listProjectMembers", "project"),
    ("addProjectMember", "project"),
    ("removeProjectMember", "project"),
    ("listUserProjects", "user"),
    // pipeline
    ("createPipeline", "project"),
    ("readPipeline", "pipeline"),
    ("updatePipeline", "pipeline"),
    ("deletePipeline", "pipeline"),
    ("runPipeline", "pipeline"),
    ("executeJob", "pipeline"),
    ("listPipelines", "system"),
    ("listPipelinesByProject", "project"),
    ("listPipelinesByOrganization", "organization"),
    // job
    ("createJob", "system"),
    ("readJob", "job"),
    ("writeJob", "job"),
    ("deleteJob", "job"),
    ("listJobs", "system"),
    ("listJobsByPipeline", "pipeline"),
    ("listJobsByProject", "project"),
    ("listJobsByOrganization", "organization"),
    ("readJobLogs", "job"),
    ("writeJobLogs", "job"),
    ("writeJobStatus", "job"),
    ("writeJobLog", "job"),
    // app
    ("createApp", "organization"),
    ("readApp", "app"),
    ("deleteApp", "app"),
    ("listAppsByOrganization", "organization"),
    // agent
    ("createAgent", "organization"),
    ("listAgents", "organization"),
    ("readAgent", "app"),
    ("readAgentStats", "app"),
    ("deleteAgent", "app"),
    // grants / policies
    ("manageGrants", "system"),
    ("manageOrgGrants", "organization"),
    ("manageProjectGrants", "project"),
    ("managePolicies", "system"),
    ("manageRoles", "system"),
];

/// Whether `key` is a permission the system knows (a [`PERMISSION_CATALOG`] key).
/// Used to validate a direct permission grant before persisting it.
#[must_use]
pub fn is_known_permission(key: &str) -> bool {
    PERMISSION_CATALOG.iter().any(|(k, _)| *k == key)
}

#[cfg(test)]
mod catalog_tests {
    use super::{PERMISSION_CATALOG, RESOURCE_TYPES};
    use std::collections::HashSet;

    #[test]
    fn permission_catalog_is_consistent() {
        let mut keys = HashSet::new();
        for (key, resource_type) in PERMISSION_CATALOG {
            assert!(
                keys.insert(*key),
                "duplicate permission key in catalog: {key}"
            );
            assert!(
                RESOURCE_TYPES.contains(resource_type),
                "permission {key} has unknown resource type {resource_type}",
            );
        }
    }
}
