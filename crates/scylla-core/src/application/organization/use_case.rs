use crate::application::authz::grant::{
    Grant, GrantPrincipal, GrantScope, ORGANIZATION_ADMIN_ROLE,
};
use crate::application::authz::policy::PolicyControl;
use crate::application::caller::CallerContext;
use crate::application::{
    OrganizationRepository, PermissionService, UserOrganizationRepository, UserRepository,
};
use crate::domain::entities::{Organization, OrganizationId, User, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::action::Action;
use crate::domain::value_objects::organization::{OrganizationDescription, OrganizationName};
use crate::domain::value_objects::role::name::RoleName;
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
    PC: PolicyControl,
> {
    org_repo: Arc<O>,
    user_org_repo: Arc<UO>,
    user_repo: Arc<U>,
    permission_service: Arc<PS>,
    policy_control: Arc<PC>,
}

impl<
    O: OrganizationRepository,
    UO: UserOrganizationRepository,
    U: UserRepository,
    PS: PermissionService,
    PC: PolicyControl,
> OrganizationUseCases<O, UO, U, PS, PC>
{
    #[instrument(skip(self, caller), fields(name = %name))]
    pub async fn create(
        &self,
        caller: &CallerContext,
        name: OrganizationName,
        description: Option<OrganizationDescription>,
    ) -> DomainResult<Organization> {
        self.permission_service
            .check(caller, Action::CreateOrganization)
            .await?;

        if self.org_repo.name_exists(&name).await? {
            return Err(DomainError::conflict("Organization name already exists"));
        }

        let org = Organization::create(name, description)?;

        // Enroll the human creator as a member + org-admin of the org they just
        // created — mirrors signup's `provision_account`. The org insert,
        // membership and owner grant happen in ONE transaction so a partial
        // failure can never leave an org without an owner. Machine/anonymous
        // callers create nothing to enroll, so they just get the bare org.
        match caller {
            CallerContext::User(user_id) => {
                let grant = Grant::new(
                    GrantPrincipal::User(user_id.clone()),
                    RoleName::new(ORGANIZATION_ADMIN_ROLE)?,
                    GrantScope::Organization(org.id().clone()),
                );
                self.org_repo
                    .provision_with_owner(&org, user_id, &grant)
                    .await?;
                // Make the org-admin grant live now so the creator can act on
                // the org immediately, without a control-plane restart.
                self.policy_control.reload().await?;
            }
            _ => {
                self.org_repo.create(&org).await?;
            }
        }

        Ok(org)
    }

    #[instrument(skip(self, caller), fields(org_id = %id))]
    pub async fn get(
        &self,
        caller: &CallerContext,
        id: &OrganizationId,
    ) -> DomainResult<Organization> {
        self.permission_service
            .check(caller, Action::ReadOrganization(id.clone()))
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
            .check(caller, Action::UpdateOrganization(id.clone()))
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
            .check(caller, Action::UpdateOrganization(id.clone()))
            .await?;
        let mut org = self.org_repo.find_by_id(id).await?;
        org.toggle_active()?;
        self.org_repo.update(&org).await?;
        Ok(())
    }

    #[instrument(skip(self, caller), fields(org_id = %id))]
    pub async fn delete(&self, caller: &CallerContext, id: &OrganizationId) -> DomainResult<()> {
        self.permission_service
            .check(caller, Action::DeleteOrganization(id.clone()))
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
            .check(caller, Action::ListOrganizations)
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
            .check(caller, Action::ListOrganizationMembers(org_id.clone()))
            .await?;

        let paginated = self.user_org_repo.list_members(org_id, pagination).await?;
        let (user_ids, metadata) = paginated.into_parts();

        // One batched read instead of N `find_by_id`; re-order to the paginated
        // membership order (the batch result order is unspecified).
        let mut by_id: std::collections::HashMap<String, User> = self
            .user_repo
            .find_by_ids(&user_ids)
            .await?
            .into_iter()
            .map(|u| (u.id().as_str().to_owned(), u))
            .collect();
        let users = user_ids
            .iter()
            .filter_map(|id| by_id.remove(id.as_str()))
            .collect();

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
            .check(caller, Action::ListUserOrganizations(user_id.clone()))
            .await?;

        let paginated = self
            .user_org_repo
            .list_user_organizations(user_id, pagination)
            .await?;
        let (org_ids, metadata) = paginated.into_parts();

        let mut by_id: std::collections::HashMap<String, Organization> = self
            .org_repo
            .find_by_ids(&org_ids)
            .await?
            .into_iter()
            .map(|o| (o.id().as_str().to_owned(), o))
            .collect();
        let orgs = org_ids
            .iter()
            .filter_map(|id| by_id.remove(id.as_str()))
            .collect();

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
            .check(caller, Action::AddOrganizationMember(org_id.clone()))
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
            .check(caller, Action::RemoveOrganizationMember(org_id.clone()))
            .await?;
        self.user_org_repo.remove_member(user_id, org_id).await
    }
}
