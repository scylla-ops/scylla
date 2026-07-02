use super::ResourceRef;
use crate::domain::entities::{AppId, JobId, OrganizationId, PipelineId, ProjectId, UserId};
use std::sync::LazyLock;

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
    /// An agent App executing the jobs of a pipeline (distinct from a user
    /// triggering a run via `RunPipeline`).
    ExecuteJob(PipelineId),
    /// Manage a pipeline's triggers (create / update / delete / enable). Pipeline-
    /// scoped like `UpdatePipeline`; firing itself is gated separately by
    /// `RunPipeline`, so managing triggers does not by itself confer run rights.
    ManageTriggers(PipelineId),
    ListPipelines,
    ListPipelinesByProject(ProjectId),
    ListPipelinesByOrganization(OrganizationId),

    // ── secret (project-scoped) ────────────────────────────────────────
    // Manage a project's secrets. Create/list/delete are gated at the project
    // scope; secret values are never read back, only referenced from pipelines.
    CreateSecret(ProjectId),
    ListSecrets(ProjectId),
    DeleteSecret(ProjectId),

    // ── job ────────────────────────────────────────────────────────────
    CreateJob,
    ReadJob(JobId),
    UpdateJob(JobId),
    DeleteJob(JobId),
    ListJobs,
    ListJobsByPipeline(PipelineId),
    ListJobsByProject(ProjectId),
    ListJobsByOrganization(OrganizationId),
    ReadJobLogs(JobId),
    /// Recorder path: create a job's log lines (User / Service only).
    WriteJobLogs(JobId),
    /// An agent App reporting a job's status while it runs.
    WriteJobStatus(JobId),
    /// An agent App appending a single log line over its stream — kept distinct
    /// from the recorder's `WriteJobLogs` so an agent can't take that path.
    AppendJobLog(JobId),

    // ── app (machine principals; an agent is a specialized app) ────────
    CreateApp(OrganizationId),
    ReadApp(AppId),
    /// Run stats of an app — how an agent's job-execution stats are read.
    ReadAppStats(AppId),
    DeleteApp(AppId),
    ListAppsByOrganization(OrganizationId),

    // ── agent (specialized apps that run jobs) ─────────────────────────
    // Reading, deleting and reading stats of an agent reuse the App-targeted
    // permissions above (an agent IS an app); only provisioning and listing
    // agents are agent-specific.
    CreateAgent(OrganizationId),
    ListAgents(OrganizationId),

    // ── grants / policies / roles ──────────────────────────────────────
    // One grant-management permission per scope (`manage<Scope>Grants`); the
    // UI presents them as a single "manage grants" concept. They stay separate
    // because each pins a different Cedar resource type (the anti-escalation
    // fence) — see the comment on `key()` below.
    /// System-scoped grant management (admin / service): manage any grant.
    ManageSystemGrants,
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
            Self::ManageTriggers(_) => "manageTriggers",
            Self::ListPipelines => "listPipelines",
            Self::ListPipelinesByProject(_) => "listPipelinesByProject",
            Self::ListPipelinesByOrganization(_) => "listPipelinesByOrganization",

            Self::CreateSecret(_) => "createSecret",
            Self::ListSecrets(_) => "listSecrets",
            Self::DeleteSecret(_) => "deleteSecret",

            Self::CreateJob => "createJob",
            Self::ReadJob(_) => "readJob",
            Self::UpdateJob(_) => "updateJob",
            Self::DeleteJob(_) => "deleteJob",
            Self::ListJobs => "listJobs",
            Self::ListJobsByPipeline(_) => "listJobsByPipeline",
            Self::ListJobsByProject(_) => "listJobsByProject",
            Self::ListJobsByOrganization(_) => "listJobsByOrganization",
            Self::ReadJobLogs(_) => "readJobLogs",
            Self::WriteJobLogs(_) => "writeJobLogs",
            Self::WriteJobStatus(_) => "writeJobStatus",
            Self::AppendJobLog(_) => "appendJobLog",

            Self::CreateApp(_) => "createApp",
            Self::ReadApp(_) => "readApp",
            Self::ReadAppStats(_) => "readAppStats",
            Self::DeleteApp(_) => "deleteApp",
            Self::ListAppsByOrganization(_) => "listAppsByOrganization",

            Self::CreateAgent(_) => "createAgent",
            Self::ListAgents(_) => "listAgents",

            // Distinct action ids per scope so the Cedar schema pins `appliesTo`
            // (System / Organization / Project) per action. A single shared
            // action would let one over-broad permit on it authorize all three
            // scopes (scope load-bearing only via the resource arm), so the
            // split is the anti-escalation fence — uniform `manage<Scope>Grants`.
            Self::ManageSystemGrants => "manageSystemGrants",
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
            | Self::ManageSystemGrants
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
            | Self::CreateSecret(id)
            | Self::ListSecrets(id)
            | Self::DeleteSecret(id)
            | Self::ManageProjectGrants(id) => ResourceRef::Project(id.clone()),

            // Pipeline-targeted
            Self::ReadPipeline(id)
            | Self::UpdatePipeline(id)
            | Self::DeletePipeline(id)
            | Self::RunPipeline(id)
            | Self::ExecuteJob(id)
            | Self::ManageTriggers(id)
            | Self::ListJobsByPipeline(id) => ResourceRef::Pipeline(id.clone()),

            // Job-targeted
            Self::ReadJob(id)
            | Self::UpdateJob(id)
            | Self::DeleteJob(id)
            | Self::ReadJobLogs(id)
            | Self::WriteJobLogs(id)
            | Self::WriteJobStatus(id)
            | Self::AppendJobLog(id) => ResourceRef::Job(id.clone()),

            Self::ReadApp(id) | Self::ReadAppStats(id) | Self::DeleteApp(id) => {
                ResourceRef::App(id.clone())
            }
        }
    }

    /// The resource-type tag this permission targets (`"user"`, `"job"`, …),
    /// derived from [`Self::resource`] so it can never drift from the actual
    /// Cedar target. Lets the authz layer place a permission within the scope
    /// hierarchy (e.g. reject a `system`-targeted permission in a project-scoped
    /// role) without a hand-maintained `(key, resource_type)` table.
    #[must_use]
    pub fn resource_type(&self) -> &'static str {
        self.resource().kind()
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

/// One sample of every [`Permission`] variant — the single enumeration of the
/// catalog. Ids are placeholders: [`Permission::key`] and
/// [`Permission::resource_type`] read only the variant, never the id value. The
/// proto-sync test (`grpc::convert`) asserts this stays a total mirror of the
/// gRPC `Permission` enum, so a forgotten variant is caught.
fn catalog_variants() -> Vec<Permission> {
    let user = UserId::new("_");
    let org = OrganizationId::new("_");
    let project = ProjectId::new("_");
    let pipeline = PipelineId::new("_");
    let job = JobId::new("_");
    let app = AppId::new("_");
    vec![
        // user
        Permission::CreateUser,
        Permission::ReadUser(user.clone()),
        Permission::UpdateUser(user.clone()),
        Permission::DeleteUser(user.clone()),
        Permission::ListUsers,
        // organization
        Permission::CreateOrganization,
        Permission::ReadOrganization(org.clone()),
        Permission::UpdateOrganization(org.clone()),
        Permission::DeleteOrganization(org.clone()),
        Permission::ListOrganizations,
        Permission::ListOrganizationMembers(org.clone()),
        Permission::AddOrganizationMember(org.clone()),
        Permission::RemoveOrganizationMember(org.clone()),
        Permission::ManageInvitations(org.clone()),
        Permission::ListUserOrganizations(user.clone()),
        // project
        Permission::CreateProject(org.clone()),
        Permission::ReadProject(project.clone()),
        Permission::UpdateProject(project.clone()),
        Permission::DeleteProject(project.clone()),
        Permission::ListProjects,
        Permission::ListProjectsByOrganization(org.clone()),
        Permission::ListProjectMembers(project.clone()),
        Permission::AddProjectMember(project.clone()),
        Permission::RemoveProjectMember(project.clone()),
        Permission::ListUserProjects(user.clone()),
        // pipeline
        Permission::CreatePipeline(project.clone()),
        Permission::ReadPipeline(pipeline.clone()),
        Permission::UpdatePipeline(pipeline.clone()),
        Permission::DeletePipeline(pipeline.clone()),
        Permission::RunPipeline(pipeline.clone()),
        Permission::ExecuteJob(pipeline.clone()),
        Permission::ManageTriggers(pipeline.clone()),
        Permission::ListPipelines,
        Permission::ListPipelinesByProject(project.clone()),
        Permission::ListPipelinesByOrganization(org.clone()),
        // secret
        Permission::CreateSecret(project.clone()),
        Permission::ListSecrets(project.clone()),
        Permission::DeleteSecret(project.clone()),
        // job
        Permission::CreateJob,
        Permission::ReadJob(job.clone()),
        Permission::UpdateJob(job.clone()),
        Permission::DeleteJob(job.clone()),
        Permission::ListJobs,
        Permission::ListJobsByPipeline(pipeline.clone()),
        Permission::ListJobsByProject(project.clone()),
        Permission::ListJobsByOrganization(org.clone()),
        Permission::ReadJobLogs(job.clone()),
        Permission::WriteJobLogs(job.clone()),
        Permission::WriteJobStatus(job.clone()),
        Permission::AppendJobLog(job),
        // app (an agent is a specialized app: its read/delete/stats live here)
        Permission::CreateApp(org.clone()),
        Permission::ReadApp(app.clone()),
        Permission::ReadAppStats(app.clone()),
        Permission::DeleteApp(app),
        Permission::ListAppsByOrganization(org.clone()),
        // agent
        Permission::CreateAgent(org.clone()),
        Permission::ListAgents(org.clone()),
        // grants / policies / roles
        Permission::ManageSystemGrants,
        Permission::ManageOrgGrants(org.clone()),
        Permission::ManageProjectGrants(project),
        Permission::ManagePolicies,
        Permission::ManageRoles,
    ]
}

/// The full authorization vocabulary: every permission key paired with the
/// resource type it targets. Drives `ListAuthzVocabulary` and grant/role
/// validation. **Derived** from the [`Permission`] enum — `key()` gives the id,
/// `resource_type()` the target type — so the resource type can never drift from
/// the actual Cedar target (no hand-maintained second column). One row per
/// [`catalog_variants`] entry.
pub static PERMISSION_CATALOG: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
    catalog_variants()
        .iter()
        .map(|p| (p.key(), p.resource_type()))
        .collect()
});

/// Whether `key` is a permission the system knows (a [`PERMISSION_CATALOG`] key).
/// Used to validate a direct permission grant before persisting it.
#[must_use]
pub fn is_known_permission(key: &str) -> bool {
    PERMISSION_CATALOG.iter().any(|(k, _)| *k == key)
}

/// The resource type tag a permission targets (a [`RESOURCE_TYPES`] entry), or
/// `None` if `key` is not a known permission. Lets the authz layer place a
/// permission within the scope hierarchy (e.g. to reject a `system`-targeted
/// permission in an organization- or project-scoped role).
#[must_use]
pub fn permission_resource_type(key: &str) -> Option<&'static str> {
    PERMISSION_CATALOG
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, rt)| *rt)
}

#[cfg(test)]
mod catalog_tests {
    use super::{PERMISSION_CATALOG, RESOURCE_TYPES, catalog_variants};
    use std::collections::HashSet;

    #[test]
    fn permission_catalog_is_consistent() {
        let mut keys = HashSet::new();
        for (key, resource_type) in PERMISSION_CATALOG.iter() {
            assert!(
                keys.insert(*key),
                "duplicate permission key in catalog: {key}"
            );
            assert!(
                RESOURCE_TYPES.contains(resource_type),
                "permission {key} has unknown resource type {resource_type}",
            );
        }
        // The derived catalog has exactly one row per enumerated variant.
        assert_eq!(PERMISSION_CATALOG.len(), catalog_variants().len());
    }
}
