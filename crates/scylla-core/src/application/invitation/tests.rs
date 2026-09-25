//! The invitation's actions through the engine, on stub ports.

use super::*;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::{InvitationId, OrganizationId, UserId};
use crate::domain::invitation::{Invitation, InvitationStatus};
use crate::domain::organization::{Organization, OrganizationName};
use crate::domain::permission::Permission;
use crate::domain::role::RoleName;
use crate::domain::user::{Email, User};
use crate::test_support::authz::{DenyingPermissionService, RecordingPermissionService, actions};
use crate::test_support::organizations::OrgBuilder;
use crate::test_support::stubs::{alice, empty_page};
use async_trait::async_trait;
use scylla_auth::authz::{Grant, PermissionService, Role, RoleRepository, ScopeKind};
use scylla_extension::Actions;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
struct StubInvitations {
    rows: Mutex<HashMap<InvitationId, Invitation>>,
}

#[async_trait]
impl InvitationRepository for StubInvitations {
    async fn create(&self, invite: &Invitation) -> DomainResult<()> {
        self.rows
            .lock()
            .unwrap()
            .insert(invite.id().clone(), invite.clone());
        Ok(())
    }
    async fn find_by_id(&self, id: &InvitationId) -> DomainResult<Invitation> {
        self.rows
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| DomainError::not_found("Invitation", id.to_string()))
    }
    async fn find_by_token(&self, _: &str) -> DomainResult<Invitation> {
        unreachable!("no invitation action reads by token")
    }
    async fn list_pending(&self, org_id: &OrganizationId) -> DomainResult<Vec<Invitation>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .values()
            .filter(|i| i.organization_id() == org_id)
            .cloned()
            .collect())
    }
    async fn revoke(&self, id: &InvitationId) -> DomainResult<()> {
        self.rows.lock().unwrap().remove(id);
        Ok(())
    }
    async fn accept_atomic(
        &self,
        _: &InvitationId,
        _: Option<&User>,
        _: &UserId,
        _: &Grant,
    ) -> DomainResult<()> {
        unreachable!("no invitation action accepts")
    }
}

struct StubOrganizations(Organization);

#[async_trait]
impl OrganizationRepository for StubOrganizations {
    async fn create(&self, organization: &Organization) -> DomainResult<Organization> {
        Ok(organization.clone())
    }
    async fn provision_with_owner(&self, _: &Organization, _: &Grant) -> DomainResult<()> {
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
        if id == self.0.id() {
            Ok(self.0.clone())
        } else {
            Err(DomainError::not_found("Organization", id.to_string()))
        }
    }
    async fn update(&self, organization: &Organization) -> DomainResult<Organization> {
        Ok(organization.clone())
    }
    async fn delete(&self, _: &OrganizationId) -> DomainResult<()> {
        Ok(())
    }
    async fn list_all(
        &self,
        _: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<Organization>> {
        empty_page()
    }
    async fn name_exists(&self, _: &OrganizationName) -> DomainResult<bool> {
        Ok(false)
    }
}

#[derive(Default)]
struct StubRoles {
    lookups: Mutex<usize>,
}

#[async_trait]
impl RoleRepository for StubRoles {
    async fn list_all(&self) -> DomainResult<Vec<Role>> {
        Ok(Vec::new())
    }
    async fn get(&self, id: &str) -> DomainResult<Option<Role>> {
        *self.lookups.lock().unwrap() += 1;
        Ok((id == "organization-admin").then(|| Role {
            id: id.to_string(),
            key: Some(id.to_string()),
            name: id.to_string(),
            description: String::new(),
            scope: ScopeKind::Organization,
            owner_org: None,
            builtin: true,
            permissions: Vec::new(),
        }))
    }
    async fn create(&self, _: &Role) -> DomainResult<()> {
        Ok(())
    }
    async fn update(&self, _: &Role) -> DomainResult<()> {
        Ok(())
    }
    async fn delete(&self, _: &str) -> DomainResult<()> {
        Ok(())
    }
}

#[derive(Default)]
struct StubMailer {
    sent: Mutex<Vec<(Email, String)>>,
    failing: bool,
}

#[async_trait]
impl Mailer for StubMailer {
    async fn send(&self, to: &Email, _: &str, body: &str) -> DomainResult<()> {
        self.sent
            .lock()
            .unwrap()
            .push((to.clone(), body.to_string()));
        if self.failing {
            return Err(DomainError::internal("smtp down"));
        }
        Ok(())
    }
}

struct Lab {
    actions: Actions,
    uc: InvitationUseCases,
    invitations: Arc<StubInvitations>,
    roles: Arc<StubRoles>,
    mailer: Arc<StubMailer>,
}

impl Lab {
    async fn create(&self, cmd: CreateInvitation) -> DomainResult<Invitation> {
        self.actions.run(&self.uc, &alice(), cmd).await
    }

    async fn revoke(&self, id: &InvitationId) -> DomainResult<Invitation> {
        self.actions
            .run(&self.uc, &alice(), RevokeInvitation { id: id.clone() })
            .await
    }
}

fn lab(permissions: Arc<dyn PermissionService>) -> Lab {
    lab_with(permissions, StubMailer::default())
}

fn lab_with(permissions: Arc<dyn PermissionService>, mailer: StubMailer) -> Lab {
    let invitations = Arc::new(StubInvitations::default());
    let roles = Arc::new(StubRoles::default());
    let mailer = Arc::new(mailer);
    Lab {
        actions: actions(permissions),
        uc: InvitationUseCases::new(
            invitations.clone(),
            Arc::new(StubOrganizations(
                OrgBuilder::new("Acme").id(organization()).build(),
            )),
            roles.clone(),
            mailer.clone(),
        ),
        invitations,
        roles,
        mailer,
    }
}

fn organization() -> OrganizationId {
    OrganizationId::new("org-1")
}

fn create(role: Option<&str>) -> CreateInvitation {
    CreateInvitation {
        organization_id: organization(),
        email: Email::new("newbie@example.com").unwrap(),
        role: role.map(|r| RoleName::new(r).unwrap()),
    }
}

#[tokio::test]
async fn a_create_checks_the_permission_then_stores_and_mails_the_invitation() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());

    let invite = lab
        .create(create(Some("organization-admin")))
        .await
        .unwrap();

    assert_eq!(
        permissions.permissions(),
        vec![Permission::ManageInvitations(organization())]
    );
    assert_eq!(invite.invited_by(), &UserId::new("alice"));
    assert!(
        lab.invitations
            .rows
            .lock()
            .unwrap()
            .contains_key(invite.id())
    );
    let sent = lab.mailer.sent.lock().unwrap();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].0.as_str(), "newbie@example.com");
    assert!(sent[0].1.contains("Acme"));
    assert!(sent[0].1.contains(invite.token()));
}

#[tokio::test]
async fn a_denied_create_never_reads_stores_or_mails() {
    let lab = lab(Arc::new(DenyingPermissionService::new()));

    let err = lab
        .create(create(Some("organization-admin")))
        .await
        .unwrap_err();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert_eq!(*lab.roles.lookups.lock().unwrap(), 0);
    assert!(lab.invitations.rows.lock().unwrap().is_empty());
    assert!(lab.mailer.sent.lock().unwrap().is_empty());
}

#[tokio::test]
async fn an_unknown_role_is_refused_before_anything_is_stored() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));

    let err = lab.create(create(Some("no-such-role"))).await.unwrap_err();

    assert!(matches!(err, DomainError::Validation(_)));
    assert!(lab.invitations.rows.lock().unwrap().is_empty());
    assert!(lab.mailer.sent.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_failed_mail_keeps_the_stored_invitation() {
    let lab = lab_with(
        Arc::new(RecordingPermissionService::new()),
        StubMailer {
            failing: true,
            ..StubMailer::default()
        },
    );

    let invite = lab.create(create(None)).await.unwrap();

    assert!(
        lab.invitations
            .rows
            .lock()
            .unwrap()
            .contains_key(invite.id())
    );
    assert_eq!(lab.mailer.sent.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn a_list_checks_its_permission_and_reads_the_pending_invitations() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let created = lab.create(create(None)).await.unwrap();

    let invitations = lab
        .actions
        .run(
            &lab.uc,
            &alice(),
            ListInvitations {
                organization_id: organization(),
            },
        )
        .await
        .unwrap();

    assert_eq!(invitations.len(), 1);
    assert_eq!(invitations[0].id(), created.id());
    assert_eq!(
        permissions.permissions()[1],
        Permission::ManageInvitations(organization())
    );
}

#[tokio::test]
async fn a_revoke_checks_the_permission_on_the_invitation_then_revokes_it() {
    let permissions = Arc::new(RecordingPermissionService::new());
    let lab = lab(permissions.clone());
    let created = lab.create(create(None)).await.unwrap();

    let revoked = lab.revoke(created.id()).await.unwrap();

    assert_eq!(revoked.id(), created.id());
    assert_eq!(revoked.status(), InvitationStatus::Revoked);
    assert_eq!(
        permissions.permissions()[1],
        Permission::RevokeInvitation(created.id().clone())
    );
    assert!(lab.invitations.rows.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_denied_revoke_never_reads_or_revokes() {
    let lab = lab(Arc::new(DenyingPermissionService::new()));
    let invitation = Invitation::create(
        organization(),
        Email::new("newbie@example.com").unwrap(),
        None,
        UserId::new("alice"),
        "token".to_string(),
    );
    lab.invitations.create(&invitation).await.unwrap();

    let err = lab.revoke(invitation.id()).await.unwrap_err();

    assert!(matches!(err, DomainError::Forbidden(_)));
    assert!(
        lab.invitations
            .rows
            .lock()
            .unwrap()
            .contains_key(invitation.id())
    );
}

#[tokio::test]
async fn an_allowed_revoke_of_an_unknown_invitation_is_not_found() {
    let lab = lab(Arc::new(RecordingPermissionService::new()));

    let err = lab.revoke(&InvitationId::new("missing")).await.unwrap_err();

    assert!(matches!(err, DomainError::NotFound { .. }));
}
