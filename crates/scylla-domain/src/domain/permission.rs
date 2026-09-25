mod resource_ref;

pub use resource_ref::*;

use crate::domain::ids::{
    AppCredentialId, AppId, InvitationId, JobId, OrganizationId, PipelineId, ProjectId, SecretId,
    TriggerId, UserId,
};
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Permission {
    CreateUser,
    ReadUser(UserId),
    UpdateUser(UserId),
    DeleteUser(UserId),
    ListUsers,

    CreateOrganization,
    ReadOrganization(OrganizationId),
    UpdateOrganization(OrganizationId),
    DeleteOrganization(OrganizationId),
    ListOrganizations,
    ListOrganizationMembers(OrganizationId),
    /// Separate from member listing so a plain member cannot enumerate invitee emails.
    ManageInvitations(OrganizationId),
    /// `manageInvitations` on the invitation's organization. Shares its key, so it is not in the catalog.
    RevokeInvitation(InvitationId),
    ListUserOrganizations(UserId),

    CreateProject(OrganizationId),
    ReadProject(ProjectId),
    UpdateProject(ProjectId),
    DeleteProject(ProjectId),
    ListProjects,
    ListProjectsByOrganization(OrganizationId),
    ListProjectMembers(ProjectId),
    ListUserProjects(UserId),

    CreatePipeline(ProjectId),
    ReadPipeline(PipelineId),
    UpdatePipeline(PipelineId),
    DeletePipeline(PipelineId),
    RunPipeline(PipelineId),
    ExecuteJob(PipelineId),
    /// Does not confer `RunPipeline`.
    ManageTriggers(PipelineId),
    /// `manageTriggers` on the trigger's pipeline. Shares its key, so it is not in the catalog.
    ManageTrigger(TriggerId),
    /// `runPipeline` on the trigger's pipeline. Shares its key, so it is not in the catalog.
    RunTriggerPipeline(TriggerId),
    ListPipelines,
    ListPipelinesByProject(ProjectId),
    ListPipelinesByOrganization(OrganizationId),

    CreateSecret(ProjectId),
    ListSecrets(ProjectId),
    DeleteSecret(SecretId),

    CreateJob,
    ReadJob(JobId),
    UpdateJob(JobId),
    DeleteJob(JobId),
    ListJobs,
    ListJobsByPipeline(PipelineId),
    ListJobsByProject(ProjectId),
    ListJobsByOrganization(OrganizationId),
    ReadJobLogs(JobId),
    WriteJobLogs(JobId),
    WriteJobStatus(JobId),
    /// Distinct from `WriteJobLogs` so an agent cannot take the recorder path.
    AppendJobLog(JobId),

    CreateApp(OrganizationId),
    ReadApp(AppId),
    ReadAppStats(AppId),
    DeleteApp(AppId),
    /// `deleteApp` on the secret's app. Shares its key, so it is not in the catalog.
    ManageAppSecret(AppCredentialId),
    ListAppsByOrganization(OrganizationId),

    CreateAgent(OrganizationId),
    ListAgents(OrganizationId),

    ManageSystemGrants,
    ManageOrgGrants(OrganizationId),
    ManageProjectGrants(ProjectId),
    ManageRoles,
}

impl Permission {
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
            Self::ManageInvitations(_) | Self::RevokeInvitation(_) => "manageInvitations",
            Self::ListUserOrganizations(_) => "listUserOrganizations",

            Self::CreateProject(_) => "createProject",
            Self::ReadProject(_) => "readProject",
            Self::UpdateProject(_) => "updateProject",
            Self::DeleteProject(_) => "deleteProject",
            Self::ListProjects => "listProjects",
            Self::ListProjectsByOrganization(_) => "listProjectsByOrganization",
            Self::ListProjectMembers(_) => "listProjectMembers",
            Self::ListUserProjects(_) => "listUserProjects",

            Self::CreatePipeline(_) => "createPipeline",
            Self::ReadPipeline(_) => "readPipeline",
            Self::UpdatePipeline(_) => "updatePipeline",
            Self::DeletePipeline(_) => "deletePipeline",
            Self::RunPipeline(_) | Self::RunTriggerPipeline(_) => "runPipeline",
            Self::ExecuteJob(_) => "executeJob",
            Self::ManageTriggers(_) | Self::ManageTrigger(_) => "manageTriggers",
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
            Self::DeleteApp(_) | Self::ManageAppSecret(_) => "deleteApp",
            Self::ListAppsByOrganization(_) => "listAppsByOrganization",

            Self::CreateAgent(_) => "createAgent",
            Self::ListAgents(_) => "listAgents",

            // One action per scope so the Cedar schema pins `appliesTo`; a shared action would let one permit cover all three.
            Self::ManageSystemGrants => "manageSystemGrants",
            Self::ManageOrgGrants(_) => "manageOrgGrants",
            Self::ManageProjectGrants(_) => "manageProjectGrants",
            Self::ManageRoles => "manageRoles",
        }
    }

    #[must_use]
    pub fn resource(&self) -> ResourceRef {
        match self {
            Self::CreateUser
            | Self::ListUsers
            | Self::CreateOrganization
            | Self::ListOrganizations
            | Self::ListProjects
            | Self::ListPipelines
            | Self::CreateJob
            | Self::ListJobs
            | Self::ManageSystemGrants
            | Self::ManageRoles => ResourceRef::System,

            Self::ReadUser(id)
            | Self::UpdateUser(id)
            | Self::DeleteUser(id)
            | Self::ListUserOrganizations(id)
            | Self::ListUserProjects(id) => ResourceRef::User(id.clone()),

            Self::ReadOrganization(id)
            | Self::UpdateOrganization(id)
            | Self::DeleteOrganization(id)
            | Self::ListOrganizationMembers(id)
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

            Self::RevokeInvitation(id) => ResourceRef::Invitation(id.clone()),

            Self::ReadProject(id)
            | Self::UpdateProject(id)
            | Self::DeleteProject(id)
            | Self::ListProjectMembers(id)
            | Self::CreatePipeline(id)
            | Self::ListPipelinesByProject(id)
            | Self::ListJobsByProject(id)
            | Self::CreateSecret(id)
            | Self::ListSecrets(id)
            | Self::ManageProjectGrants(id) => ResourceRef::Project(id.clone()),

            Self::ReadPipeline(id)
            | Self::UpdatePipeline(id)
            | Self::DeletePipeline(id)
            | Self::RunPipeline(id)
            | Self::ExecuteJob(id)
            | Self::ManageTriggers(id)
            | Self::ListJobsByPipeline(id) => ResourceRef::Pipeline(id.clone()),

            Self::ReadJob(id)
            | Self::UpdateJob(id)
            | Self::DeleteJob(id)
            | Self::ReadJobLogs(id)
            | Self::WriteJobLogs(id)
            | Self::WriteJobStatus(id)
            | Self::AppendJobLog(id) => ResourceRef::Job(id.clone()),

            Self::DeleteSecret(id) => ResourceRef::Secret(id.clone()),

            Self::ManageTrigger(id) | Self::RunTriggerPipeline(id) => {
                ResourceRef::Trigger(id.clone())
            }

            Self::ReadApp(id) | Self::ReadAppStats(id) | Self::DeleteApp(id) => {
                ResourceRef::App(id.clone())
            }

            Self::ManageAppSecret(id) => ResourceRef::AppSecret(id.clone()),
        }
    }

    #[must_use]
    pub fn resource_type(&self) -> &'static str {
        self.resource().kind()
    }
}

pub const RESOURCE_TYPES: &[&str] = &[
    "system",
    "user",
    "organization",
    "invitation",
    "project",
    "pipeline",
    "job",
    "secret",
    "trigger",
    "app",
    "app_secret",
];

fn catalog_variants() -> Vec<Permission> {
    let user = UserId::new("_");
    let org = OrganizationId::new("_");
    let project = ProjectId::new("_");
    let pipeline = PipelineId::new("_");
    let job = JobId::new("_");
    let secret = SecretId::new("_");
    let app = AppId::new("_");
    vec![
        Permission::CreateUser,
        Permission::ReadUser(user.clone()),
        Permission::UpdateUser(user.clone()),
        Permission::DeleteUser(user.clone()),
        Permission::ListUsers,
        Permission::CreateOrganization,
        Permission::ReadOrganization(org.clone()),
        Permission::UpdateOrganization(org.clone()),
        Permission::DeleteOrganization(org.clone()),
        Permission::ListOrganizations,
        Permission::ListOrganizationMembers(org.clone()),
        Permission::ManageInvitations(org.clone()),
        Permission::ListUserOrganizations(user.clone()),
        Permission::CreateProject(org.clone()),
        Permission::ReadProject(project.clone()),
        Permission::UpdateProject(project.clone()),
        Permission::DeleteProject(project.clone()),
        Permission::ListProjects,
        Permission::ListProjectsByOrganization(org.clone()),
        Permission::ListProjectMembers(project.clone()),
        Permission::ListUserProjects(user.clone()),
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
        Permission::CreateSecret(project.clone()),
        Permission::ListSecrets(project.clone()),
        Permission::DeleteSecret(secret),
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
        Permission::CreateApp(org.clone()),
        Permission::ReadApp(app.clone()),
        Permission::ReadAppStats(app.clone()),
        Permission::DeleteApp(app),
        Permission::ListAppsByOrganization(org.clone()),
        Permission::CreateAgent(org.clone()),
        Permission::ListAgents(org.clone()),
        Permission::ManageSystemGrants,
        Permission::ManageOrgGrants(org.clone()),
        Permission::ManageProjectGrants(project),
        Permission::ManageRoles,
    ]
}

pub static PERMISSION_CATALOG: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
    catalog_variants()
        .iter()
        .map(|p| (p.key(), p.resource_type()))
        .collect()
});

#[must_use]
pub fn is_known_permission(key: &str) -> bool {
    PERMISSION_CATALOG.iter().any(|(k, _)| *k == key)
}

#[must_use]
pub fn permission_resource_type(key: &str) -> Option<&'static str> {
    PERMISSION_CATALOG
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, rt)| *rt)
}

#[cfg(test)]
mod catalog_tests {
    use super::{
        PERMISSION_CATALOG, Permission, RESOURCE_TYPES, catalog_variants, is_known_permission,
    };
    use crate::domain::ids::{AppCredentialId, InvitationId, TriggerId};
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
        assert_eq!(PERMISSION_CATALOG.len(), catalog_variants().len());
    }

    #[test]
    fn trigger_permissions_reuse_catalog_keys() {
        let trigger = TriggerId::new("_");
        for permission in [
            Permission::ManageTrigger(trigger.clone()),
            Permission::RunTriggerPipeline(trigger),
        ] {
            assert!(is_known_permission(permission.key()));
        }
    }

    #[test]
    fn invitation_permissions_reuse_catalog_keys() {
        assert!(is_known_permission(
            Permission::RevokeInvitation(InvitationId::new("_")).key()
        ));
    }

    #[test]
    fn app_secret_permissions_reuse_catalog_keys() {
        assert!(is_known_permission(
            Permission::ManageAppSecret(AppCredentialId::new("_")).key()
        ));
    }
}
