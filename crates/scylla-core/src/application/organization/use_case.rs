use crate::application::{OrganizationRepository, UserOrganizationRepository, UserRepository};
use crate::domain::entities::{Organization, OrganizationId, User, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::organization::{OrganizationDescription, OrganizationName};
use crate::domain::value_objects::{PaginatedResult, PaginationMetadata, PaginationParams};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct OrganizationUseCases<
    O: OrganizationRepository,
    UO: UserOrganizationRepository,
    U: UserRepository,
> {
    org_repo: Arc<O>,
    user_org_repo: Arc<UO>,
    user_repo: Arc<U>,
}

impl<O: OrganizationRepository, UO: UserOrganizationRepository, U: UserRepository>
    OrganizationUseCases<O, UO, U>
{
    #[instrument(skip(self), fields(name = %name))]
    pub async fn create(
        &self,
        name: OrganizationName,
        description: Option<OrganizationDescription>,
    ) -> DomainResult<Organization> {
        if self.org_repo.name_exists(&name).await? {
            return Err(DomainError::conflict("Organization name already exists"));
        }

        let org = Organization::create(name, description)?;
        self.org_repo.create(&org).await
    }

    #[instrument(skip(self), fields(org_id = %id))]
    pub async fn get(&self, id: &OrganizationId) -> DomainResult<Organization> {
        self.org_repo.find_by_id(id).await
    }

    #[instrument(skip(self), fields(org_id = %id))]
    pub async fn update(
        &self,
        id: &OrganizationId,
        name: Option<OrganizationName>,
        description: Option<Option<OrganizationDescription>>,
    ) -> DomainResult<Organization> {
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

    #[instrument(skip(self), fields(org_id = %id))]
    pub async fn toggle_active(&self, id: &OrganizationId) -> DomainResult<()> {
        let mut org = self.org_repo.find_by_id(id).await?;
        org.toggle_active()?;
        self.org_repo.update(&org).await?;
        Ok(())
    }

    #[instrument(skip(self), fields(org_id = %id))]
    pub async fn delete(&self, id: &OrganizationId) -> DomainResult<()> {
        self.org_repo.find_by_id(id).await?;
        self.org_repo.delete(id).await
    }

    #[instrument(skip(self))]
    pub async fn list(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>> {
        self.org_repo.list_all(pagination).await
    }

    #[instrument(skip(self), fields(org_id = %org_id))]
    pub async fn list_users(
        &self,
        org_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<(Vec<User>, PaginationMetadata)> {
        let paginated = self.user_org_repo.list_members(org_id, pagination).await?;
        let (user_ids, metadata) = paginated.into_parts();

        let mut users = Vec::with_capacity(user_ids.len());
        for user_id in &user_ids {
            let user = self.user_repo.find_by_id(user_id).await?;
            users.push(user);
        }

        Ok((users, metadata))
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn list_user_orgs(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<(Vec<Organization>, PaginationMetadata)> {
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

    #[instrument(skip(self), fields(user_id = %user_id, org_id = %org_id))]
    pub async fn add_user(&self, user_id: &UserId, org_id: &OrganizationId) -> DomainResult<()> {
        if self.user_org_repo.is_member(user_id, org_id).await? {
            return Err(DomainError::conflict(
                "User is already a member of this organization",
            ));
        }

        self.user_org_repo.add_member(user_id, org_id).await
    }

    #[instrument(skip(self), fields(user_id = %user_id, org_id = %org_id))]
    pub async fn remove_user(&self, user_id: &UserId, org_id: &OrganizationId) -> DomainResult<()> {
        self.user_org_repo.remove_member(user_id, org_id).await
    }
}
