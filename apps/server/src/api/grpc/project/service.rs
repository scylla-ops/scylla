use crate::api::grpc::organization::repos::OrganizationRepository;
use crate::api::grpc::project::models::{
    InsertableProject, Project, ProjectPatch, UserProjectRelation,
};
use crate::api::grpc::project::repos::ProjectRepository;
use crate::api::grpc::user::models::User;
use crate::api::grpc::user::repos::UserRepository;
use derive_more::Constructor;
use surrealdb::RecordIdKey;
use thiserror::Error;

#[derive(Constructor)]
pub struct ProjectService<R: ProjectRepository, UR: UserRepository, OR: OrganizationRepository> {
    _marker: std::marker::PhantomData<(R, UR, OR)>,
}

#[derive(Debug, Error)]
pub enum ProjectDomainError {
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Invalid pagination parameter: {field}")]
    InvalidPagination { field: &'static str },
    #[error("Project not found")]
    ProjectNotFound,
    #[error("User not found")]
    UserNotFound,
    #[error("Organization not found")]
    OrganizationNotFound,
    #[error("Project name already exists in this organization")]
    ProjectNameExistsInOrg,
    #[error("User is not a member of the project's organization")]
    UserNotInOrganization,
    #[error(transparent)]
    Repo(#[from] anyhow::Error),
}

impl<R: ProjectRepository, UR: UserRepository, OR: OrganizationRepository>
    ProjectService<R, UR, OR>
{
    pub async fn create_project(
        name: String,
        description: Option<String>,
        org_id: RecordIdKey,
    ) -> Result<Project, ProjectDomainError> {
        // verify organization exists
        let _org = OR::get_organization_by_id(org_id.clone())
            .await?
            .ok_or(ProjectDomainError::OrganizationNotFound)?;

        // check if project name already exists in this organization
        if let Some(_) = R::get_project_by_name_and_org(name.clone(), org_id.clone()).await? {
            return Err(ProjectDomainError::ProjectNameExistsInOrg);
        }

        let project = InsertableProject {
            name,
            description,
            organization: crate::api::grpc::tables::organizations::to_record_id(org_id),
        };

        R::create_project(project)
            .await
            .map_err(ProjectDomainError::Repo)
    }

    pub async fn get_project(project_id: RecordIdKey) -> Result<Project, ProjectDomainError> {
        let opt = R::get_project_by_id(project_id).await?;
        match opt {
            Some(project) => Ok(project),
            None => Err(ProjectDomainError::ProjectNotFound),
        }
    }

    pub async fn list_projects(
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Project>, usize), ProjectDomainError> {
        const MAX_PAGE_SIZE: u32 = 100;
        if page == 0 {
            return Err(ProjectDomainError::InvalidPagination { field: "page" });
        }
        if page_size == 0 {
            return Err(ProjectDomainError::InvalidPagination { field: "page_size" });
        }
        if page_size > MAX_PAGE_SIZE {
            return Err(ProjectDomainError::Validation(format!(
                "page_size must be <= {}",
                MAX_PAGE_SIZE
            )));
        }
        let limit_i64: i64 = page_size.into();
        let offset_u128 = (page as u128)
            .saturating_sub(1)
            .saturating_mul(page_size as u128);
        let offset_i64 = i64::try_from(offset_u128)
            .map_err(|_| ProjectDomainError::Validation("computed offset is too big".into()))?;
        let list = R::list_projects(limit_i64, offset_i64).await?;
        let total = list.len();
        Ok((list, total))
    }

    pub async fn list_organization_projects(
        org_id: RecordIdKey,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<Project>, usize), ProjectDomainError> {
        // verify organization exists
        let _org = OR::get_organization_by_id(org_id.clone())
            .await?
            .ok_or(ProjectDomainError::OrganizationNotFound)?;

        const MAX_PAGE_SIZE: u32 = 100;
        if page == 0 {
            return Err(ProjectDomainError::InvalidPagination { field: "page" });
        }
        if page_size == 0 {
            return Err(ProjectDomainError::InvalidPagination { field: "page_size" });
        }
        if page_size > MAX_PAGE_SIZE {
            return Err(ProjectDomainError::Validation(format!(
                "page_size must be <= {}",
                MAX_PAGE_SIZE
            )));
        }

        let limit: i64 = page_size.into();
        let offset_u128 = (page as u128)
            .saturating_sub(1)
            .saturating_mul(page_size as u128);
        let offset: i64 = i64::try_from(offset_u128)
            .map_err(|_| ProjectDomainError::Validation("computed offset is too big".into()))?;

        let list = R::list_organization_projects(org_id, limit, offset).await?;
        let total = list.len();
        Ok((list, total))
    }

    pub async fn update_project(
        project_id: RecordIdKey,
        patch: ProjectPatch,
    ) -> Result<Project, ProjectDomainError> {
        // get existing project
        let existing = R::get_project_by_id(project_id.clone())
            .await?
            .ok_or(ProjectDomainError::ProjectNotFound)?;

        // if name is being changed, check uniqueness within the organization
        if let Some(ref new_name) = patch.name {
            let org_key = existing.organization.key();
            if let Some(existing_with_name) =
                R::get_project_by_name_and_org(new_name.clone(), org_key.clone()).await?
            {
                if existing_with_name.id.key().to_string() != project_id.to_string() {
                    return Err(ProjectDomainError::ProjectNameExistsInOrg);
                }
            }
        }

        let opt = R::update_project(project_id, patch).await?;

        match opt {
            Some(project) => Ok(project),
            None => Err(ProjectDomainError::ProjectNotFound),
        }
    }

    pub async fn deactivate_project(project_id: RecordIdKey) -> Result<(), ProjectDomainError> {
        let res = R::deactivate_project(project_id).await?;
        match res {
            Some(_) => Ok(()),
            None => Err(ProjectDomainError::ProjectNotFound),
        }
    }

    pub async fn add_user_to_project(
        user_id: RecordIdKey,
        project_id: RecordIdKey,
        role: String,
    ) -> Result<UserProjectRelation, ProjectDomainError> {
        // verify user exists
        let _user = UR::get_user_by_id(user_id.to_string())
            .await?
            .ok_or(ProjectDomainError::UserNotFound)?;

        // verify project exists and get its organization
        let project = R::get_project_by_id(project_id.clone())
            .await?
            .ok_or(ProjectDomainError::ProjectNotFound)?;

        // verify user is a member of the project's organization
        let org_id = project.organization.key();
        let user_orgs = OR::list_user_organizations(user_id.clone(), 1000, 0).await?;
        let is_member = user_orgs
            .iter()
            .any(|(org, _)| org.id.key().to_string() == org_id.to_string());

        if !is_member {
            return Err(ProjectDomainError::UserNotInOrganization);
        }

        R::add_user_to_project(user_id, project_id, role)
            .await
            .map_err(ProjectDomainError::Repo)
    }

    pub async fn remove_user_from_project(
        user_id: RecordIdKey,
        project_id: RecordIdKey,
    ) -> Result<(), ProjectDomainError> {
        R::remove_user_from_project(user_id, project_id)
            .await
            .map_err(ProjectDomainError::Repo)
    }

    pub async fn list_project_users(
        project_id: RecordIdKey,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<(User, UserProjectRelation)>, usize), ProjectDomainError> {
        let _ = R::get_project_by_id(project_id.clone())
            .await?
            .ok_or(ProjectDomainError::ProjectNotFound)?;

        const MAX_PAGE_SIZE: u32 = 100;
        if page == 0 {
            return Err(ProjectDomainError::InvalidPagination { field: "page" });
        }
        if page_size == 0 {
            return Err(ProjectDomainError::InvalidPagination { field: "page_size" });
        }
        if page_size > MAX_PAGE_SIZE {
            return Err(ProjectDomainError::Validation(format!(
                "page_size must be <= {}",
                MAX_PAGE_SIZE
            )));
        }

        let limit: i64 = page_size.into();
        let offset_u128 = (page as u128)
            .saturating_sub(1)
            .saturating_mul(page_size as u128);
        let offset: i64 = i64::try_from(offset_u128)
            .map_err(|_| ProjectDomainError::Validation("computed offset is too big".into()))?;

        let list = R::list_project_users(project_id, limit, offset).await?;
        let total = list.len();
        Ok((list, total))
    }

    pub async fn list_user_projects(
        user_id: RecordIdKey,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<(Project, UserProjectRelation)>, usize), ProjectDomainError> {
        let _user = UR::get_user_by_id(user_id.to_string())
            .await?
            .ok_or(ProjectDomainError::UserNotFound)?;

        const MAX_PAGE_SIZE: u32 = 100;
        if page == 0 {
            return Err(ProjectDomainError::InvalidPagination { field: "page" });
        }
        if page_size == 0 {
            return Err(ProjectDomainError::InvalidPagination { field: "page_size" });
        }
        if page_size > MAX_PAGE_SIZE {
            return Err(ProjectDomainError::Validation(format!(
                "page_size must be <= {}",
                MAX_PAGE_SIZE
            )));
        }

        let limit: i64 = page_size.into();
        let offset_u128 = (page as u128)
            .saturating_sub(1)
            .saturating_mul(page_size as u128);
        let offset: i64 = i64::try_from(offset_u128)
            .map_err(|_| ProjectDomainError::Validation("computed offset is too big".into()))?;

        let list = R::list_user_projects(user_id, limit, offset).await?;
        let total = list.len();
        Ok((list, total))
    }
}
