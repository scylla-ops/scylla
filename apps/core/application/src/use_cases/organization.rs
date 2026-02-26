use derive_more::Constructor;
use domain::entities::{
    Organization, OrganizationId, User, UserId, UserOrganization, UserOrganizationId,
};
use domain::errors::{DomainError, DomainResult};
use domain::ports::{OrganizationRepository, UserOrganizationRepository, UserRepository};
use domain::value_objects::organization::{OrganizationDescription, OrganizationName};
use domain::value_objects::{PaginatedResult, PaginationMetadata, PaginationParams};
use std::sync::Arc;

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

    pub async fn get(&self, id: &OrganizationId) -> DomainResult<Organization> {
        self.org_repo.find_by_id(id).await
    }

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

    pub async fn toggle_active(&self, id: &OrganizationId) -> DomainResult<()> {
        let mut org = self.org_repo.find_by_id(id).await?;
        org.toggle_active()?;
        self.org_repo.update(&org).await?;
        Ok(())
    }

    pub async fn delete(&self, id: &OrganizationId) -> DomainResult<()> {
        self.org_repo.find_by_id(id).await?;
        self.org_repo.delete(id).await
    }

    pub async fn list(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>> {
        self.org_repo.list_all(pagination).await
    }

    pub async fn list_users(
        &self,
        org_id: &OrganizationId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<(Vec<(User, UserOrganization)>, PaginationMetadata)> {
        let paginated = self
            .user_org_repo
            .list_users_in_organization(org_id, pagination)
            .await?;
        let (user_ids, metadata) = paginated.into_parts();

        let mut pairs = Vec::with_capacity(user_ids.len());
        for user_id in &user_ids {
            let user = self.user_repo.find_by_id(user_id).await?;
            let membership = self
                .user_org_repo
                .find_by_user_and_organization(user_id, org_id)
                .await?;
            pairs.push((user, membership));
        }

        Ok((pairs, metadata))
    }

    pub async fn list_user_orgs(
        &self,
        user_id: &UserId,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<(Vec<Organization>, PaginationMetadata)> {
        let paginated = self
            .user_org_repo
            .list_organizations_for_user(user_id, pagination)
            .await?;
        let (org_ids, metadata) = paginated.into_parts();

        let mut orgs = Vec::with_capacity(org_ids.len());
        for org_id in &org_ids {
            let org = self.org_repo.find_by_id(org_id).await?;
            orgs.push(org);
        }

        Ok((orgs, metadata))
    }

    pub async fn add_user(
        &self,
        user_id: &UserId,
        org_id: &OrganizationId,
        role: &str,
    ) -> DomainResult<UserOrganizationId> {
        if self
            .user_org_repo
            .find_by_user_and_organization(user_id, org_id)
            .await
            .is_ok()
        {
            return Err(DomainError::conflict(
                "User is already a member of this organization",
            ));
        }

        self.user_org_repo
            .add_user_to_organization(user_id, org_id, role)
            .await
    }

    pub async fn remove_user(&self, user_id: &UserId, org_id: &OrganizationId) -> DomainResult<()> {
        self.user_org_repo
            .remove_user_from_organization(user_id, org_id)
            .await
    }
}
