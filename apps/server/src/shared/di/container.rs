/// Dependency Injection Container
///
/// This container initializes and wires up all dependencies for the application.
use crate::application::ports::{AuthService, PasswordHasher, RbacEnforcer};
use crate::application::use_cases::auth::{LoginUseCase, RevokeTokenUseCase, ValidateTokenUseCase};
use crate::application::use_cases::job::{
    CreateJobUseCase, DeleteJobUseCase, GetJobUseCase, ListJobsByPipelineUseCase,
    ListJobsByStatusUseCase, ListJobsUseCase, UpdateJobUseCase,
};
use crate::application::use_cases::orchestrator::RunPipelineUseCase;
use crate::application::use_cases::organization::{
    AddUserToOrganizationUseCase, CreateOrganizationUseCase, DeleteOrganizationUseCase,
    GetOrganizationUseCase, ListOrganizationUsersUseCase, ListOrganizationsUseCase,
    ListUserOrganizationsUseCase, RemoveUserFromOrganizationUseCase,
    ToggleActiveOrganizationUseCase, UpdateOrganizationUseCase,
};
use crate::application::use_cases::pipeline::{
    CreatePipelineUseCase, DeletePipelineUseCase, GetPipelineUseCase, ListPipelinesUseCase,
    UpdatePipelineUseCase,
};
use crate::application::use_cases::project::{
    AddUserToProjectUseCase, CreateProjectUseCase, DeleteProjectUseCase, GetProjectUseCase,
    ListProjectUsersUseCase, ListProjectsUseCase, ListUserProjectsUseCase,
    RemoveUserFromProjectUseCase, ToggleProjectActiveUseCase, UpdateProjectUseCase,
};
use crate::application::use_cases::user::{
    ChangeUserGlobalRoleUseCase, CreateUserUseCase, DeleteUserUseCase, GetUserUseCase,
    ListUsersUseCase, UpdateUserUseCase,
};
use crate::domain::repositories::{
    JobRepository, OrganizationRepository, PipelineRepository, ProjectRepository,
    UserOrganizationRepository, UserProjectRepository, UserRepository,
};
use crate::infrastructure::auth::{Argon2PasswordHasher, PasetoAuthService};
use crate::infrastructure::persistence::surrealdb::{
    SurrealJobRepository, SurrealOrganizationRepository, SurrealPipelineRepository,
    SurrealProjectRepository, SurrealUserOrganizationRepository, SurrealUserProjectRepository,
    SurrealUserRepository,
};
use crate::infrastructure::rbac::CasbinRbacEnforcer;
use casbin::Enforcer;
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// Application dependency container
pub struct AppContainer {
    // Repositories
    user_repo: Arc<dyn UserRepository>,
    organization_repo: Arc<dyn OrganizationRepository>,
    project_repo: Arc<dyn ProjectRepository>,
    pipeline_repo: Arc<dyn PipelineRepository>,
    job_repo: Arc<dyn JobRepository>,

    // Services (Ports)
    auth_service: Arc<dyn AuthService>,
    password_hasher: Arc<dyn PasswordHasher>,
    rbac_enforcer: Arc<dyn RbacEnforcer>,

    // Use Cases - User
    create_user_use_case:
        Arc<CreateUserUseCase<dyn UserRepository, dyn PasswordHasher, dyn RbacEnforcer>>,
    get_user_use_case: Arc<GetUserUseCase<dyn UserRepository>>,
    update_user_use_case: Arc<UpdateUserUseCase<dyn UserRepository>>,
    delete_user_use_case: Arc<DeleteUserUseCase<dyn UserRepository>>,
    list_users_use_case: Arc<ListUsersUseCase<dyn UserRepository>>,
    change_user_global_role_use_case:
        Arc<ChangeUserGlobalRoleUseCase<dyn UserRepository, dyn RbacEnforcer>>,

    // Use Cases - Auth
    login_use_case: Arc<LoginUseCase<dyn UserRepository, dyn PasswordHasher, dyn AuthService>>,
    validate_token_use_case: Arc<ValidateTokenUseCase<dyn AuthService>>,
    revoke_token_use_case: Arc<RevokeTokenUseCase<dyn AuthService>>,

    // Use Cases - Organization
    create_organization_use_case: Arc<
        CreateOrganizationUseCase<
            dyn OrganizationRepository,
            dyn UserOrganizationRepository,
            dyn RbacEnforcer,
        >,
    >,
    get_organization_use_case: Arc<GetOrganizationUseCase<dyn OrganizationRepository>>,
    update_organization_use_case: Arc<UpdateOrganizationUseCase<dyn OrganizationRepository>>,
    toggle_organization_active_use_case:
        Arc<ToggleActiveOrganizationUseCase<dyn OrganizationRepository>>,
    delete_organization_use_case: Arc<DeleteOrganizationUseCase<dyn OrganizationRepository>>,
    list_organizations_use_case: Arc<ListOrganizationsUseCase<dyn OrganizationRepository>>,
    list_organization_users_use_case:
        Arc<ListOrganizationUsersUseCase<dyn UserOrganizationRepository, dyn UserRepository>>,
    list_user_organizations_use_case: Arc<
        ListUserOrganizationsUseCase<dyn UserOrganizationRepository, dyn OrganizationRepository>,
    >,
    add_user_to_organization_use_case: Arc<
        AddUserToOrganizationUseCase<
            dyn UserOrganizationRepository,
            dyn UserRepository,
            dyn OrganizationRepository,
            dyn RbacEnforcer,
        >,
    >,
    remove_user_from_organization_use_case:
        Arc<RemoveUserFromOrganizationUseCase<dyn UserOrganizationRepository>>,

    // Use Cases - Project
    create_project_use_case: Arc<
        CreateProjectUseCase<dyn ProjectRepository, dyn UserProjectRepository, dyn RbacEnforcer>,
    >,
    get_project_use_case: Arc<GetProjectUseCase<dyn ProjectRepository>>,
    update_project_use_case: Arc<UpdateProjectUseCase<dyn ProjectRepository>>,
    toggle_project_active_use_case: Arc<ToggleProjectActiveUseCase<dyn ProjectRepository>>,
    delete_project_use_case: Arc<DeleteProjectUseCase<dyn ProjectRepository>>,
    list_projects_use_case: Arc<ListProjectsUseCase<dyn ProjectRepository>>,
    list_project_users_use_case:
        Arc<ListProjectUsersUseCase<dyn UserProjectRepository, dyn UserRepository>>,
    list_user_projects_use_case:
        Arc<ListUserProjectsUseCase<dyn UserProjectRepository, dyn ProjectRepository>>,
    add_user_to_project_use_case:
        Arc<AddUserToProjectUseCase<dyn UserProjectRepository, dyn RbacEnforcer>>,
    remove_user_from_project_use_case: Arc<RemoveUserFromProjectUseCase<dyn UserProjectRepository>>,

    // Use Cases - Pipeline
    create_pipeline_use_case: Arc<CreatePipelineUseCase<dyn PipelineRepository>>,
    get_pipeline_use_case: Arc<GetPipelineUseCase<dyn PipelineRepository>>,
    update_pipeline_use_case: Arc<UpdatePipelineUseCase<dyn PipelineRepository>>,
    delete_pipeline_use_case: Arc<DeletePipelineUseCase<dyn PipelineRepository>>,
    list_pipelines_use_case: Arc<ListPipelinesUseCase<dyn PipelineRepository>>,

    // Use Cases - Job
    create_job_use_case: Arc<CreateJobUseCase<dyn JobRepository, dyn PipelineRepository>>,
    get_job_use_case: Arc<GetJobUseCase<dyn JobRepository>>,
    update_job_use_case: Arc<UpdateJobUseCase<dyn JobRepository>>,
    delete_job_use_case: Arc<DeleteJobUseCase<dyn JobRepository>>,
    list_jobs_use_case: Arc<ListJobsUseCase<dyn JobRepository>>,
    list_jobs_by_status_use_case: Arc<ListJobsByStatusUseCase<dyn JobRepository>>,
    list_jobs_by_pipeline_use_case: Arc<ListJobsByPipelineUseCase<dyn JobRepository>>,

    // Use Cases - Orchestrator
    run_pipeline_use_case: Arc<RunPipelineUseCase<dyn JobRepository, dyn PipelineRepository>>,
}

impl AppContainer {
    pub fn new(
        db: Arc<Surreal<Any>>,
        enforcer: Enforcer,
        auth_config: &crate::config::AuthConfig,
    ) -> anyhow::Result<Self> {
        // Initialize repositories
        let user_repo: Arc<dyn UserRepository> = Arc::new(SurrealUserRepository::new(db.clone()));
        let organization_repo: Arc<dyn OrganizationRepository> =
            Arc::new(SurrealOrganizationRepository::new(db.clone()));
        let project_repo: Arc<dyn ProjectRepository> =
            Arc::new(SurrealProjectRepository::new(db.clone()));
        let pipeline_repo: Arc<dyn PipelineRepository> =
            Arc::new(SurrealPipelineRepository::new(db.clone()));
        let job_repo: Arc<dyn JobRepository> = Arc::new(SurrealJobRepository::new(db.clone()));
        let user_organization_repo: Arc<dyn UserOrganizationRepository> =
            Arc::new(SurrealUserOrganizationRepository::new(db.clone()));
        let user_project_repo: Arc<dyn UserProjectRepository> =
            Arc::new(SurrealUserProjectRepository::new(db.clone()));

        // Initialize services
        let auth_service: Arc<dyn AuthService> =
            Arc::new(PasetoAuthService::from_config(auth_config).map_err(|e| {
                anyhow::anyhow!("Failed to create auth service from config: {}", e)
            })?);
        let password_hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2PasswordHasher::default());
        let rbac_enforcer: Arc<dyn RbacEnforcer> = Arc::new(CasbinRbacEnforcer::new(enforcer));

        // Initialize use cases - User
        let create_user_use_case = Arc::new(CreateUserUseCase::new(
            user_repo.clone(),
            password_hasher.clone(),
            rbac_enforcer.clone(),
        ));
        let get_user_use_case = Arc::new(GetUserUseCase::new(user_repo.clone()));
        let update_user_use_case = Arc::new(UpdateUserUseCase::new(user_repo.clone()));
        let delete_user_use_case = Arc::new(DeleteUserUseCase::new(user_repo.clone()));
        let list_users_use_case = Arc::new(ListUsersUseCase::new(user_repo.clone()));
        let change_user_global_role_use_case = Arc::new(ChangeUserGlobalRoleUseCase::new(
            user_repo.clone(),
            rbac_enforcer.clone(),
        ));

        // Initialize use cases - Auth
        let login_use_case = Arc::new(LoginUseCase::new(
            user_repo.clone(),
            password_hasher.clone(),
            auth_service.clone(),
        ));
        let validate_token_use_case = Arc::new(ValidateTokenUseCase::new(auth_service.clone()));
        let revoke_token_use_case = Arc::new(RevokeTokenUseCase::new(auth_service.clone()));

        // Initialize use cases - Organization
        let create_organization_use_case = Arc::new(CreateOrganizationUseCase::new(
            organization_repo.clone(),
            user_organization_repo.clone(),
            rbac_enforcer.clone(),
        ));
        let get_organization_use_case =
            Arc::new(GetOrganizationUseCase::new(organization_repo.clone()));
        let update_organization_use_case =
            Arc::new(UpdateOrganizationUseCase::new(organization_repo.clone()));
        let toggle_organization_active_use_case = Arc::new(ToggleActiveOrganizationUseCase::new(
            organization_repo.clone(),
        ));
        let delete_organization_use_case =
            Arc::new(DeleteOrganizationUseCase::new(organization_repo.clone()));
        let list_organizations_use_case =
            Arc::new(ListOrganizationsUseCase::new(organization_repo.clone()));
        let list_organization_users_use_case = Arc::new(ListOrganizationUsersUseCase::new(
            user_organization_repo.clone(),
            user_repo.clone(),
        ));
        let list_user_organizations_use_case = Arc::new(ListUserOrganizationsUseCase::new(
            user_organization_repo.clone(),
            organization_repo.clone(),
        ));
        let add_user_to_organization_use_case = Arc::new(AddUserToOrganizationUseCase::new(
            user_organization_repo.clone(),
            user_repo.clone(),
            organization_repo.clone(),
            rbac_enforcer.clone(),
        ));
        let remove_user_from_organization_use_case = Arc::new(
            RemoveUserFromOrganizationUseCase::new(user_organization_repo.clone()),
        );

        // Initialize use cases - Project
        let create_project_use_case = Arc::new(CreateProjectUseCase::new(
            project_repo.clone(),
            user_project_repo.clone(),
            rbac_enforcer.clone(),
        ));
        let get_project_use_case = Arc::new(GetProjectUseCase::new(project_repo.clone()));
        let update_project_use_case = Arc::new(UpdateProjectUseCase::new(project_repo.clone()));
        let toggle_project_active_use_case =
            Arc::new(ToggleProjectActiveUseCase::new(project_repo.clone()));
        let delete_project_use_case = Arc::new(DeleteProjectUseCase::new(project_repo.clone()));
        let list_projects_use_case = Arc::new(ListProjectsUseCase::new(project_repo.clone()));
        let list_project_users_use_case = Arc::new(ListProjectUsersUseCase::new(
            user_project_repo.clone(),
            user_repo.clone(),
        ));
        let list_user_projects_use_case = Arc::new(ListUserProjectsUseCase::new(
            user_project_repo.clone(),
            project_repo.clone(),
        ));
        let add_user_to_project_use_case = Arc::new(AddUserToProjectUseCase::new(
            user_project_repo.clone(),
            rbac_enforcer.clone(),
        ));
        let remove_user_from_project_use_case =
            Arc::new(RemoveUserFromProjectUseCase::new(user_project_repo.clone()));

        // Initialize use cases - Pipeline
        let create_pipeline_use_case = Arc::new(CreatePipelineUseCase::new(pipeline_repo.clone()));
        let get_pipeline_use_case = Arc::new(GetPipelineUseCase::new(pipeline_repo.clone()));
        let update_pipeline_use_case = Arc::new(UpdatePipelineUseCase::new(pipeline_repo.clone()));
        let delete_pipeline_use_case = Arc::new(DeletePipelineUseCase::new(pipeline_repo.clone()));
        let list_pipelines_use_case = Arc::new(ListPipelinesUseCase::new(pipeline_repo.clone()));

        // Initialize use cases - Job
        let create_job_use_case = Arc::new(CreateJobUseCase::new(
            job_repo.clone(),
            pipeline_repo.clone(),
        ));
        let get_job_use_case = Arc::new(GetJobUseCase::new(job_repo.clone()));
        let update_job_use_case = Arc::new(UpdateJobUseCase::new(job_repo.clone()));
        let delete_job_use_case = Arc::new(DeleteJobUseCase::new(job_repo.clone()));
        let list_jobs_use_case = Arc::new(ListJobsUseCase::new(job_repo.clone()));
        let list_jobs_by_status_use_case = Arc::new(ListJobsByStatusUseCase::new(job_repo.clone()));
        let list_jobs_by_pipeline_use_case =
            Arc::new(ListJobsByPipelineUseCase::new(job_repo.clone()));

        // Initialize use cases - Orchestrator
        let run_pipeline_use_case = Arc::new(RunPipelineUseCase::new(
            job_repo.clone(),
            pipeline_repo.clone(),
        ));

        Ok(Self {
            user_repo,
            organization_repo,
            project_repo,
            pipeline_repo,
            job_repo,
            auth_service,
            password_hasher,
            rbac_enforcer,
            create_user_use_case,
            get_user_use_case,
            update_user_use_case,
            delete_user_use_case,
            list_users_use_case,
            change_user_global_role_use_case,
            login_use_case,
            validate_token_use_case,
            revoke_token_use_case,
            create_organization_use_case,
            get_organization_use_case,
            update_organization_use_case,
            toggle_organization_active_use_case,
            delete_organization_use_case,
            list_organizations_use_case,
            list_organization_users_use_case,
            list_user_organizations_use_case,
            add_user_to_organization_use_case,
            remove_user_from_organization_use_case,
            create_project_use_case,
            get_project_use_case,
            update_project_use_case,
            toggle_project_active_use_case,
            delete_project_use_case,
            list_projects_use_case,
            list_project_users_use_case,
            list_user_projects_use_case,
            add_user_to_project_use_case,
            remove_user_from_project_use_case,
            create_pipeline_use_case,
            get_pipeline_use_case,
            update_pipeline_use_case,
            delete_pipeline_use_case,
            list_pipelines_use_case,
            create_job_use_case,
            get_job_use_case,
            update_job_use_case,
            delete_job_use_case,
            list_jobs_use_case,
            list_jobs_by_status_use_case,
            list_jobs_by_pipeline_use_case,
            run_pipeline_use_case,
        })
    }

    // Getters for repositories
    pub fn user_repo(&self) -> Arc<dyn UserRepository> {
        self.user_repo.clone()
    }

    pub fn organization_repo(&self) -> Arc<dyn OrganizationRepository> {
        self.organization_repo.clone()
    }

    pub fn project_repo(&self) -> Arc<dyn ProjectRepository> {
        self.project_repo.clone()
    }

    pub fn pipeline_repo(&self) -> Arc<dyn PipelineRepository> {
        self.pipeline_repo.clone()
    }

    pub fn job_repo(&self) -> Arc<dyn JobRepository> {
        self.job_repo.clone()
    }

    // Getters for services
    pub fn auth_service(&self) -> Arc<dyn AuthService> {
        self.auth_service.clone()
    }

    pub fn password_hasher(&self) -> Arc<dyn PasswordHasher> {
        self.password_hasher.clone()
    }

    pub fn rbac_enforcer(&self) -> Arc<dyn RbacEnforcer> {
        self.rbac_enforcer.clone()
    }

    // Getters for use cases - User
    pub fn create_user_use_case(
        &self,
    ) -> Arc<CreateUserUseCase<dyn UserRepository, dyn PasswordHasher, dyn RbacEnforcer>> {
        self.create_user_use_case.clone()
    }

    pub fn get_user_use_case(&self) -> Arc<GetUserUseCase<dyn UserRepository>> {
        self.get_user_use_case.clone()
    }

    pub fn update_user_use_case(&self) -> Arc<UpdateUserUseCase<dyn UserRepository>> {
        self.update_user_use_case.clone()
    }

    pub fn delete_user_use_case(&self) -> Arc<DeleteUserUseCase<dyn UserRepository>> {
        self.delete_user_use_case.clone()
    }

    pub fn list_users_use_case(&self) -> Arc<ListUsersUseCase<dyn UserRepository>> {
        self.list_users_use_case.clone()
    }

    pub fn change_user_global_role_use_case(
        &self,
    ) -> Arc<ChangeUserGlobalRoleUseCase<dyn UserRepository, dyn RbacEnforcer>> {
        self.change_user_global_role_use_case.clone()
    }

    // Getters for use cases - Auth
    pub fn login_use_case(
        &self,
    ) -> Arc<LoginUseCase<dyn UserRepository, dyn PasswordHasher, dyn AuthService>> {
        self.login_use_case.clone()
    }

    pub fn validate_token_use_case(&self) -> Arc<ValidateTokenUseCase<dyn AuthService>> {
        self.validate_token_use_case.clone()
    }

    pub fn revoke_token_use_case(&self) -> Arc<RevokeTokenUseCase<dyn AuthService>> {
        self.revoke_token_use_case.clone()
    }

    // Getters for use cases - Organization
    pub fn create_organization_use_case(
        &self,
    ) -> Arc<
        CreateOrganizationUseCase<
            dyn OrganizationRepository,
            dyn UserOrganizationRepository,
            dyn RbacEnforcer,
        >,
    > {
        self.create_organization_use_case.clone()
    }

    pub fn get_organization_use_case(
        &self,
    ) -> Arc<GetOrganizationUseCase<dyn OrganizationRepository>> {
        self.get_organization_use_case.clone()
    }

    pub fn update_organization_use_case(
        &self,
    ) -> Arc<UpdateOrganizationUseCase<dyn OrganizationRepository>> {
        self.update_organization_use_case.clone()
    }

    pub fn toggle_organization_active_use_case(
        &self,
    ) -> Arc<ToggleActiveOrganizationUseCase<dyn OrganizationRepository>> {
        self.toggle_organization_active_use_case.clone()
    }

    pub fn delete_organization_use_case(
        &self,
    ) -> Arc<DeleteOrganizationUseCase<dyn OrganizationRepository>> {
        self.delete_organization_use_case.clone()
    }

    pub fn list_organizations_use_case(
        &self,
    ) -> Arc<ListOrganizationsUseCase<dyn OrganizationRepository>> {
        self.list_organizations_use_case.clone()
    }

    pub fn list_organization_users_use_case(
        &self,
    ) -> Arc<ListOrganizationUsersUseCase<dyn UserOrganizationRepository, dyn UserRepository>> {
        self.list_organization_users_use_case.clone()
    }

    pub fn list_user_organizations_use_case(
        &self,
    ) -> Arc<ListUserOrganizationsUseCase<dyn UserOrganizationRepository, dyn OrganizationRepository>>
    {
        self.list_user_organizations_use_case.clone()
    }

    pub fn add_user_to_organization_use_case(
        &self,
    ) -> Arc<
        AddUserToOrganizationUseCase<
            dyn UserOrganizationRepository,
            dyn UserRepository,
            dyn OrganizationRepository,
            dyn RbacEnforcer,
        >,
    > {
        self.add_user_to_organization_use_case.clone()
    }

    pub fn remove_user_from_organization_use_case(
        &self,
    ) -> Arc<RemoveUserFromOrganizationUseCase<dyn UserOrganizationRepository>> {
        self.remove_user_from_organization_use_case.clone()
    }

    // Getters for use cases - Project
    pub fn create_project_use_case(
        &self,
    ) -> Arc<CreateProjectUseCase<dyn ProjectRepository, dyn UserProjectRepository, dyn RbacEnforcer>>
    {
        self.create_project_use_case.clone()
    }

    pub fn get_project_use_case(&self) -> Arc<GetProjectUseCase<dyn ProjectRepository>> {
        self.get_project_use_case.clone()
    }

    pub fn update_project_use_case(&self) -> Arc<UpdateProjectUseCase<dyn ProjectRepository>> {
        self.update_project_use_case.clone()
    }

    pub fn toggle_project_active_use_case(
        &self,
    ) -> Arc<ToggleProjectActiveUseCase<dyn ProjectRepository>> {
        self.toggle_project_active_use_case.clone()
    }

    pub fn delete_project_use_case(&self) -> Arc<DeleteProjectUseCase<dyn ProjectRepository>> {
        self.delete_project_use_case.clone()
    }

    pub fn list_projects_use_case(&self) -> Arc<ListProjectsUseCase<dyn ProjectRepository>> {
        self.list_projects_use_case.clone()
    }

    pub fn list_project_users_use_case(
        &self,
    ) -> Arc<ListProjectUsersUseCase<dyn UserProjectRepository, dyn UserRepository>> {
        self.list_project_users_use_case.clone()
    }

    pub fn list_user_projects_use_case(
        &self,
    ) -> Arc<ListUserProjectsUseCase<dyn UserProjectRepository, dyn ProjectRepository>> {
        self.list_user_projects_use_case.clone()
    }

    pub fn add_user_to_project_use_case(
        &self,
    ) -> Arc<AddUserToProjectUseCase<dyn UserProjectRepository, dyn RbacEnforcer>> {
        self.add_user_to_project_use_case.clone()
    }

    pub fn remove_user_from_project_use_case(
        &self,
    ) -> Arc<RemoveUserFromProjectUseCase<dyn UserProjectRepository>> {
        self.remove_user_from_project_use_case.clone()
    }

    // Getters for use cases - Pipeline
    pub fn create_pipeline_use_case(&self) -> Arc<CreatePipelineUseCase<dyn PipelineRepository>> {
        self.create_pipeline_use_case.clone()
    }

    pub fn get_pipeline_use_case(&self) -> Arc<GetPipelineUseCase<dyn PipelineRepository>> {
        self.get_pipeline_use_case.clone()
    }

    pub fn update_pipeline_use_case(&self) -> Arc<UpdatePipelineUseCase<dyn PipelineRepository>> {
        self.update_pipeline_use_case.clone()
    }

    pub fn delete_pipeline_use_case(&self) -> Arc<DeletePipelineUseCase<dyn PipelineRepository>> {
        self.delete_pipeline_use_case.clone()
    }

    pub fn list_pipelines_use_case(&self) -> Arc<ListPipelinesUseCase<dyn PipelineRepository>> {
        self.list_pipelines_use_case.clone()
    }

    // Getters for use cases - Job
    pub fn create_job_use_case(
        &self,
    ) -> Arc<CreateJobUseCase<dyn JobRepository, dyn PipelineRepository>> {
        self.create_job_use_case.clone()
    }

    pub fn get_job_use_case(&self) -> Arc<GetJobUseCase<dyn JobRepository>> {
        self.get_job_use_case.clone()
    }

    pub fn update_job_use_case(&self) -> Arc<UpdateJobUseCase<dyn JobRepository>> {
        self.update_job_use_case.clone()
    }

    pub fn delete_job_use_case(&self) -> Arc<DeleteJobUseCase<dyn JobRepository>> {
        self.delete_job_use_case.clone()
    }

    pub fn list_jobs_use_case(&self) -> Arc<ListJobsUseCase<dyn JobRepository>> {
        self.list_jobs_use_case.clone()
    }

    pub fn list_jobs_by_status_use_case(&self) -> Arc<ListJobsByStatusUseCase<dyn JobRepository>> {
        self.list_jobs_by_status_use_case.clone()
    }

    pub fn list_jobs_by_pipeline_use_case(
        &self,
    ) -> Arc<ListJobsByPipelineUseCase<dyn JobRepository>> {
        self.list_jobs_by_pipeline_use_case.clone()
    }

    // Getters for use cases - Orchestrator
    pub fn run_pipeline_use_case(
        &self,
    ) -> Arc<RunPipelineUseCase<dyn JobRepository, dyn PipelineRepository>> {
        self.run_pipeline_use_case.clone()
    }
}
