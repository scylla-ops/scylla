use crate::application::caller::CallerContext;
use crate::application::{
    OrganizationRepository, PermissionService, UserOrganizationRepository, UserRepository,
};
use crate::domain::entities::{Organization, OrganizationId, User, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::organization::{OrganizationDescription, OrganizationName};
use crate::domain::value_objects::permission::Permission;
use crate::domain::value_objects::{PaginatedResult, PaginationMetadata, PaginationParams};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct OrganizationUseCases<
    O: OrganizationRepository,
    UO: UserOrganizationRepository,
    U: UserRepository,
    PS: PermissionService,
> {
    org_repo: Arc<O>,
    user_org_repo: Arc<UO>,
    user_repo: Arc<U>,
    permission_service: Arc<PS>,
}

impl<
    O: OrganizationRepository,
    UO: UserOrganizationRepository,
    U: UserRepository,
    PS: PermissionService,
> OrganizationUseCases<O, UO, U, PS>
{
    #[instrument(skip(self, caller), fields(name = %name))]
    pub async fn create(
        &self,
        caller: &CallerContext,
        name: OrganizationName,
        description: Option<OrganizationDescription>,
    ) -> DomainResult<Organization> {
        self.permission_service
            .check(caller, Permission::CreateOrganization)
            .await?;

        if self.org_repo.name_exists(&name).await? {
            return Err(DomainError::conflict("Organization name already exists"));
        }

        let org = Organization::create(name, description)?;
        self.org_repo.create(&org).await
    }

    #[instrument(skip(self, caller), fields(org_id = %id))]
    pub async fn get(
        &self,
        caller: &CallerContext,
        id: &OrganizationId,
    ) -> DomainResult<Organization> {
        self.permission_service
            .check(caller, Permission::ReadOrganization(id.clone()))
            .await?;
        self.org_repo.find_by_id(id).await
    }

    #[instrument(skip(self, caller), fields(org_id = %id))]
    pub async fn update(
        &self,
        caller: &CallerContext,
        id: &OrganizationId,
        name: Option<OrganizationName>,
        description: Option<Option<OrganizationDescription>>,
    ) -> DomainResult<Organization> {
        self.permission_service
            .check(caller, Permission::UpdateOrganization(id.clone()))
            .await?;

        let mut org = self.org_repo.find_by_id(id).await?;

        if let Some(new_name) = name {
            if self.org_repo.name_exists(&new_name).await? && org.name() != &new_name {
                return Err(DomainError::conflict("Organization name already exists"));
            }
            org.update_name(new_name)?;
        }
        if let Some(new_desc) = description {
            org.update_description(new_desc)?;
        }

        self.org_repo.update(&org).await
    }

    #[instrument(skip(self, caller), fields(org_id = %id))]
    pub async fn toggle_active(
        &self,
        caller: &CallerContext,
        id: &OrganizationId,
    ) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::UpdateOrganization(id.clone()))
            .await?;
        let mut org = self.org_repo.find_by_id(id).await?;
        org.toggle_active()?;
        self.org_repo.update(&org).await?;
        Ok(())
    }

    #[instrument(skip(self, caller), fields(org_id = %id))]
    pub async fn delete(&self, caller: &CallerContext, id: &OrganizationId) -> DomainResult<()> {
        self.permission_service
            .check(caller, Permission::DeleteOrganization(id.clone()))
            .await?;
        self.org_repo.find_by_id(id).await?;
        self.org_repo.delete(id).await
    }

    #[instrument(skip(self, caller))]
    pub async fn list(
        &self,
        caller: &CallerContext,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>> {
        self.permission_service
            .check(caller, Permission::ListOrganizations)
            .await?;
        self.org_repo.list_all(pagination).await
    }

    #[instrument(skip(self, caller), fields(org_id = %org_id))]
    pub async fn list_users(
        &self,
        caller: &CallerContext,
        org_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<(Vec<User>, PaginationMetadata)> {
        self.permission_service
            .check(caller, Permission::ListOrganizationMembers(org_id.clone()))
            .await?;

        let paginated = self.user_org_repo.list_members(org_id, pagination).await?;
        let (user_ids, metadata) = paginated.into_parts();

        let mut users = Vec::with_capacity(user_ids.len());
        for user_id in &user_ids {
            let user = self.user_repo.find_by_id(user_id).await?;
            users.push(user);
        }

        Ok((users, metadata))
    }

    #[instrument(skip(self, caller), fields(user_id = %user_id))]
    pub async fn list_user_orgs(
        &self,
        caller: &CallerContext,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<(Vec<Organization>, PaginationMetadata)> {
        self.permission_service
            .check(
                caller,
                Permission::ListUserOrganizations(user_id.clone()),
            )
            .await?;

        let paginated = self
            .user_org_repo
            .list_user_organizations(user_id, pagination)
            .await?;
        let (org_ids, metadata) = paginated.into_parts();

        let mut orgs = Vec::with_capacity(org_ids.len());
        for org_id in &org_ids {
            let org = self.org_repo.find_by_id(org_id).await?;
            orgs.push(org);
        }

        Ok((orgs, metadata))
    }

    #[instrument(skip(self, caller), fields(user_id = %user_id, org_id = %org_id))]
    pub async fn add_user(
        &self,
        caller: &CallerContext,
        user_id: &UserId,
        org_id: &OrganizationId,
    ) -> DomainResult<()> {
        self.permission_service
            .check(
                caller,
                Permission::AddOrganizationMember(org_id.clone()),
            )
            .await?;

        if self.user_org_repo.is_member(user_id, org_id).await? {
            return Err(DomainError::conflict(
                "User is already a member of this organization",
            ));
        }

        self.user_org_repo.add_member(user_id, org_id).await
    }

    #[instrument(skip(self, caller), fields(user_id = %user_id, org_id = %org_id))]
    pub async fn remove_user(
        &self,
        caller: &CallerContext,
        user_id: &UserId,
        org_id: &OrganizationId,
    ) -> DomainResult<()> {
        self.permission_service
            .check(
                caller,
                Permission::RemoveOrganizationMember(org_id.clone()),
            )
            .await?;
        self.user_org_repo.remove_member(user_id, org_id).await
    }
}
