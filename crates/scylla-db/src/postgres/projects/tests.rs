use super::PgProjectRepository;
use crate::domain::errors::DomainError;
use crate::postgres::PgOrganizationRepository;
use crate::test_support::prelude::*;
use scylla_auth::authz::Visibility;
use scylla_core::application::{OrganizationRepository, ProjectRepository};
use sqlx::PgPool;

#[sqlx::test(migrations = "../../migrations")]
async fn create_then_find_round_trip(pool: PgPool) {
    let org = seed_org(&pool, "acme").await;
    let repo = PgProjectRepository::new(pool);
    let project = project(&org, "rocket");

    repo.create(&project).await.expect("create");
    let found = repo.find_by_id(project.id()).await.expect("find");

    assert_eq!(found.id(), project.id());
    assert_eq!(found.organization_id(), org.id());
    assert_eq!(found.created_at(), project.created_at());
}

/// Project creation must honour the quota policy's refusal, and refuse before
/// writing. Uses a Service caller to bypass Cedar and isolate the quota check;
/// the policy is a double that allows two creations per scope and denies the
/// third.
#[sqlx::test(migrations = "../../migrations")]
async fn project_quota_enforced(pool: PgPool) {
    use crate::domain::project::ProjectName;
    use crate::postgres::{
        PgAuthzEntityProvider, PgGrantRepository, PgRoleRepository, PgUserRepository,
    };
    use scylla_auth::audit::NoopAuditLog;
    use scylla_auth::caller::CallerContext;
    use scylla_auth::caller::ServiceIdentity;
    use scylla_auth::cedar::CedarPermissionService;
    use scylla_core::application::ProjectUseCases;
    use std::sync::Arc;

    let org = seed_org(&pool, "limited").await;
    let permission = Arc::new(
        CedarPermissionService::new(
            Arc::new(PgAuthzEntityProvider::new(pool.clone())),
            Arc::new(PgRoleRepository::new(pool.clone())),
            Arc::new(PgGrantRepository::new(pool.clone())),
            Arc::new(NoopAuditLog),
        )
        .await
        .expect("cedar"),
    );
    let uc = ProjectUseCases::new(
        Arc::new(PgProjectRepository::new(pool.clone())),
        Arc::new(PgUserRepository::new(pool.clone())),
        permission.clone(),
        permission.clone(),
        permission,
        Arc::new(DenyAfter::new(2)),
    );
    let caller = CallerContext::Service(ServiceIdentity::recorder());

    for n in ["a", "b"] {
        uc.create(
            &caller,
            ProjectName::new(n).unwrap(),
            None,
            org.id().clone(),
        )
        .await
        .expect("under quota");
    }
    let err = uc
        .create(
            &caller,
            ProjectName::new("c").unwrap(),
            None,
            org.id().clone(),
        )
        .await
        .expect_err("over quota");
    assert!(matches!(err, DomainError::QuotaExceeded(_)));

    // The refusal came before any write: the organization still holds exactly
    // the two projects the policy let through.
    let listed = PgProjectRepository::new(pool.clone())
        .list_by_organization(org.id(), None, &Visibility::All)
        .await
        .expect("list projects");
    assert_eq!(listed.items().len(), 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn fk_violation_on_unknown_organization_maps_to_conflict(pool: PgPool) {
    let phantom = org("never-persisted");
    let project = project(&phantom, "orphan");
    let repo = PgProjectRepository::new(pool);

    assert!(matches!(
        repo.create(&project).await,
        Err(DomainError::Conflict(_)),
    ));
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_by_organization_filters_other_orgs(pool: PgPool) {
    let org_a = seed_org(&pool, "org-a").await;
    let org_b = seed_org(&pool, "org-b").await;
    let repo = PgProjectRepository::new(pool);

    repo.create(&project(&org_a, "a1")).await.unwrap();
    repo.create(&project(&org_a, "a2")).await.unwrap();
    repo.create(&project(&org_b, "b1")).await.unwrap();

    assert_eq!(
        repo.list_by_organization(org_a.id(), None, &Visibility::All)
            .await
            .unwrap()
            .metadata()
            .total_count(),
        2,
    );
    assert_eq!(
        repo.list_by_organization(org_b.id(), None, &Visibility::All)
            .await
            .unwrap()
            .metadata()
            .total_count(),
        1,
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_active_filters_inactive(pool: PgPool) {
    let org = seed_org(&pool, "org").await;
    let repo = PgProjectRepository::new(pool);
    repo.create(&project(&org, "active")).await.unwrap();
    repo.create(
        &ProjectBuilder::new(&org, "dormant")
            .is_active(false)
            .build(),
    )
    .await
    .unwrap();

    assert_eq!(
        repo.list_active(None)
            .await
            .unwrap()
            .metadata()
            .total_count(),
        1,
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn cascade_organization_delete_removes_projects(pool: PgPool) {
    let org = seed_org(&pool, "doomed").await;
    let project_repo = PgProjectRepository::new(pool.clone());
    let project = project(&org, "child");
    project_repo.create(&project).await.unwrap();

    PgOrganizationRepository::new(pool)
        .delete(org.id())
        .await
        .unwrap();

    assert!(matches!(
        project_repo.find_by_id(project.id()).await,
        Err(DomainError::NotFound { .. }),
    ));
}

/// H1: creating a project for a human owner must atomically write the project,
/// a `project-admin` owner grant — so a project is never left without an
/// administrator. That grant is also what puts the creator on the project:
/// there is no second row to write.
#[sqlx::test(migrations = "../../migrations")]
async fn provision_with_owner_writes_the_owner_grant(pool: PgPool) {
    use crate::domain::role::RoleName;
    use crate::postgres::PgGrantRepository;
    use scylla_auth::authz::{Grant, GrantRepository, PROJECT_ADMIN_ROLE, Principal, Scope};

    let org = seed_org(&pool, "acme").await;
    let owner = seed_user(&pool, "alice").await;
    let project = project(&org, "rocket");
    let grant = Grant::new(
        Principal::User(owner.id().clone()),
        RoleName::new(PROJECT_ADMIN_ROLE).unwrap(),
        Scope::Project(project.id().clone()),
    );

    let repo = PgProjectRepository::new(pool.clone());
    repo.provision_with_owner(&project, &grant)
        .await
        .expect("provision");

    assert_eq!(
        repo.find_by_id(project.id()).await.unwrap().id(),
        project.id()
    );
    let grants = PgGrantRepository::new(pool).list_all().await.unwrap();
    assert!(
        grants.iter().any(|g| {
            matches!(&g.principal, Principal::User(u) if u.as_str() == owner.id().as_str())
                && g.role.as_str() == PROJECT_ADMIN_ROLE
                && matches!(&g.scope, Scope::Project(p) if p.as_str() == project.id().as_str())
        }),
        "creator should hold a project-admin owner grant",
    );
}

/// The provisioning transaction is atomic: a failure on any insert (here a
/// dangling owner id → FK violation) rolls back the project too.
#[sqlx::test(migrations = "../../migrations")]
async fn provision_with_owner_rolls_back_on_failure(pool: PgPool) {
    use crate::domain::role::RoleName;
    use crate::postgres::PgGrantRepository;
    use scylla_auth::authz::{Grant, GrantRepository, PROJECT_ADMIN_ROLE, Principal, Scope};

    // Note what this no longer proves: it used to fail on a dangling owner,
    // because the membership row had a foreign key to `users`. `grants` has
    // none — `principal_id` is polymorphic — so a fabricated owner id now
    // inserts cleanly. Nothing in the API can produce one (the owner is the
    // authenticated caller), and the delete triggers clear grants when a user
    // goes, but the database no longer refuses it on its own.
    let org = seed_org(&pool, "acme").await;
    let owner = seed_user(&pool, "alice").await;
    let project = project(&org, "rocket");
    let repo = PgProjectRepository::new(pool.clone());

    // The project already exists, so the insert inside the transaction fails.
    repo.create(&project).await.expect("seed the clash");

    let grant = Grant::new(
        Principal::User(owner.id().clone()),
        RoleName::new(PROJECT_ADMIN_ROLE).unwrap(),
        Scope::Project(project.id().clone()),
    );
    assert!(
        repo.provision_with_owner(&project, &grant).await.is_err(),
        "a duplicate project id must fail the transaction"
    );

    // And the grant must not have survived the rollback.
    let grants = PgGrantRepository::new(pool).list_all().await.unwrap();
    assert!(
        grants.is_empty(),
        "the owner grant is rolled back with the project insert"
    );
}

/// The visibility rule, end to end: in an organization you see the projects you
/// hold a role on, and nothing else. This is the whole point of removing the
/// membership floor, so it is asserted against a real database rather than a
/// stub.
#[sqlx::test(migrations = "../../migrations")]
async fn a_project_listing_shows_only_what_the_caller_holds(pool: PgPool) {
    use crate::domain::role::RoleName;
    use crate::postgres::{PgAuthzEntityProvider, PgGrantRepository, PgRoleRepository};
    use scylla_auth::audit::NoopAuditLog;
    use scylla_auth::authz::{
        Grant, GrantRepository, PROJECT_VIEWER_ROLE, Principal, Scope, Visibility,
        VisibilityResolver,
    };
    use scylla_auth::caller::CallerContext;
    use scylla_auth::cedar::CedarPermissionService;
    use std::sync::Arc;

    let org = seed_org(&pool, "acme").await;
    let mine = seed_project(&pool, &org, "apollo").await;
    let _theirs = seed_project(&pool, &org, "zeus").await;
    let alice = seed_user(&pool, "alice").await;

    PgGrantRepository::new(pool.clone())
        .create(&Grant::new(
            Principal::User(alice.id().clone()),
            RoleName::new(PROJECT_VIEWER_ROLE).unwrap(),
            Scope::Project(mine.id().clone()),
        ))
        .await
        .unwrap();

    let permission = CedarPermissionService::new(
        Arc::new(PgAuthzEntityProvider::new(pool.clone())),
        Arc::new(PgRoleRepository::new(pool.clone())),
        Arc::new(PgGrantRepository::new(pool.clone())),
        Arc::new(NoopAuditLog),
    )
    .await
    .expect("cedar");

    let visible = permission
        .visible_scopes(&CallerContext::User(alice.id().clone()), "readProject")
        .await
        .expect("visible scopes");
    assert_eq!(
        visible,
        Visibility::Scoped {
            orgs: vec![],
            projects: vec![mine.id().clone()],
        }
    );

    let page = PgProjectRepository::new(pool)
        .list_by_organization(org.id(), None, &visible)
        .await
        .expect("list");
    assert_eq!(
        page.metadata().total_count(),
        1,
        "the other project is not hers"
    );
    assert_eq!(page.items()[0].id(), mine.id());
}
