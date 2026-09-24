//! The organization's actions through the engine, on stub ports.

use super::*;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{OrganizationId, UserId};
use crate::domain::organization::{Organization, OrganizationName};
use crate::domain::permission::Permission;
use crate::test_support::authz::{DenyingPermissionService, RecordingPermissionService, actions};
use crate::test_support::stubs::{CountingPolicy, NoUsers, alice, empty_page};
use async_trait::async_trait;
use scylla_auth::authz::{Grant, PermissionService};
use scylla_extension::Actions;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
struct StubOrganizations {
    rows: Mutex<HashMap<OrganizationId, Organization>>,
    grants: Mutex<Vec<Grant>>,
}

#[async_trait]
impl OrganizationRepository for StubOrganizations {
    async fn create(&self, organization: &Organization) -> DomainResult<Organization> {
        self.rows
            .lock()
            .unwrap()
            .insert(organization.id().clone(), organization.clone());
        Ok(organization.clone())
    }
    async fn provision_with_owner(
        &self,
        organization: &Organization,
        grant: &Grant,
    ) -> DomainResult<()> {
        self.create(organization).await?;
        self.grants.lock().unwrap().push(grant.clone());
        Ok(())
    }
    async fn list_principals(
        &self,
        _: &OrganizationId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<UserId>> {
        empty_page()
    }
    async fn list_for_user(
        &self,
        _: &UserId,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>> {
        empty_page()
    }
    async fn find_by_id(&self, id: &OrganizationId) -> DomainResult<Organization> {
        self.rows
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| DomainError::not_found("Organization", id.to_string()))
    }
    async fn find_by_ids(&self, _: &[OrganizationId]) -> DomainResult<Vec<Organization>> {
        Ok(Vec::new())
    }
    async fn find_by_name(&self, name: &OrganizationName) -> DomainResult<Organization> {
        Err(DomainError::not_found("Organization", name.to_string()))
    }
    async fn update(&self, organization: &Organization) -> DomainResult<Organization> {
        self.create(organization).await
    }
    async fn delete(&self, id: &OrganizationId) -> DomainResult<()> {
        self.rows.lock().unwrap().remove(id);
        Ok(())
    }
    async fn list_all(
        &self,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>> {
        empty_page()
    }
    async fn list_active(
        &self,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>> {
        empty_page()
    }
    async fn name_exists(&self, name: &OrganizationName) -> DomainResult<bool> {
        Ok(self.rows.lock().unwrap().values().any(|o| o.name() == name))
    }
}

struct Lab {
    actions: Actions,
    uc: OrganizationUseCases,
    organizations: Arc<StubOrganizations>,
    policy: Arc<CountingPolicy>,
}

impl Lab {
    async fn create(&self, name: &str) -> DomainResult<Organization> {
        self.actions.run(&self.uc, &alice(), create(name)).await
    }
}

fn lab(permissions: Arc<dyn PermissionService>) -> Lab {
    let organizations = Arc::new(StubOrganizations::default());
    let policy = Arc::new(CountingPolicy::default());
    Lab {
        actions: actions(permissions),
        uc: OrganizationUseCases::new(organizations.clone(), Arc::new(NoUsers), policy.clone()),
        organizations,
        policy,
    }
}

fn create(name: &str) -> CreateOrganization {
    CreateOrganization {
        name: OrganizationName::new(name).unwrap(),
        description: None,
    }
}

#[tokio::test]
async fn a_create_by_a_user_checks_the_permission_then_writes_the_owner_grant() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());

    let organization = lab.create("acme").await.unwrap();

    assert_eq!(
        permissions.permissions(),
        vec![Permission::CreateOrganization]
    );
    assert!(
        lab.organizations
            .rows
            .lock()
            .unwrap()
            .contains_key(organization.id())
    );
    assert_eq!(lab.organizations.grants.lock().unwrap().len(), 1);
    assert_eq!(lab.policy.reloads(), 1);
}

#[tokio::test]
async fn a_denied_caller_writes_nothing() {
    let lab = lab(Arc::new(DenyingPermissionService::new()));

    let err = lab.create("acme").await.unwrap_err();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert!(lab.organizations.rows.lock().unwrap().is_empty());
    assert_eq!(lab.policy.reloads(), 0);
}

#[tokio::test]
async fn a_taken_name_is_a_conflict_and_writes_nothing() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));
    lab.create("acme").await.unwrap();

    let err = lab.create("acme").await.unwrap_err();

    assert!(matches!(err, DomainError::Conflict(_)));
    assert_eq!(lab.organizations.rows.lock().unwrap().len(), 1);
    assert_eq!(lab.policy.reloads(), 1);
}

#[tokio::test]
async fn an_update_stages_the_change_and_persists_it() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let created = lab.create("old").await.unwrap();

    let updated = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            UpdateOrganization {
                id: created.id().clone(),
                name: Some(OrganizationName::new("new").unwrap()),
                description: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.name().as_str(), "new");
    assert_eq!(
        lab.organizations.rows.lock().unwrap()[created.id()]
            .name()
            .as_str(),
        "new"
    );
    assert_eq!(
        permissions.permissions()[1],
        Permission::UpdateOrganization(created.id().clone())
    );
}

#[tokio::test]
async fn an_update_to_its_own_name_is_not_a_conflict() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));
    let created = lab.create("acme").await.unwrap();

    lab.actions
        .run(
            &lab.uc,
            &alice(),
            UpdateOrganization {
                id: created.id().clone(),
                name: Some(OrganizationName::new("acme").unwrap()),
                description: None,
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn a_delete_returns_the_tombstone_and_reloads_the_policies() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let created = lab.create("gone").await.unwrap();

    let deleted = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            DeleteOrganization {
                id: created.id().clone(),
            },
        )
        .await
        .unwrap();

    assert_eq!(deleted.last_state().id(), created.id());
    assert!(lab.organizations.rows.lock().unwrap().is_empty());
    assert_eq!(lab.policy.reloads(), 2);
    assert_eq!(
        permissions.permissions()[1],
        Permission::DeleteOrganization(created.id().clone())
    );
}

#[tokio::test]
async fn a_read_checks_its_permission() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let created = lab.create("seen").await.unwrap();

    let read = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            GetOrganization {
                id: created.id().clone(),
            },
        )
        .await
        .unwrap();

    assert_eq!(read.id(), created.id());
    assert_eq!(
        permissions.permissions()[1],
        Permission::ReadOrganization(created.id().clone())
    );
}
