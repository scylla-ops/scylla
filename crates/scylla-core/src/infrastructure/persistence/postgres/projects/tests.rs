use super::PgProjectRepository;
use crate::application::{OrganizationRepository, ProjectRepository};
use crate::domain::errors::DomainError;
use crate::infrastructure::persistence::postgres::PgOrganizationRepository;
use crate::test_support::prelude::*;
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

#[sqlx::test(migrations = "../../migrations")]
async fn count_by_organization_reflects_inserts(pool: PgPool) {
    let org = seed_org(&pool, "counted").await;
    let repo = PgProjectRepository::new(pool);

    assert_eq!(repo.count_by_organization(org.id()).await.unwrap(), 0);
    repo.create(&project(&org, "p1")).await.expect("p1");
    repo.create(&project(&org, "p2")).await.expect("p2");
    assert_eq!(repo.count_by_organization(org.id()).await.unwrap(), 2);
}

/// Project creation must cap projects per org. Uses a Service caller to
/// bypass Cedar and isolate the quota check.
#[cfg(feature = "permission")]
#[sqlx::test(migrations = "../../migrations")]
async fn project_quota_enforced(pool: PgPool) {
    use crate::application::audit::NoopAuditLog;
    use crate::application::caller::CallerContext;
    use crate::application::{ProjectUseCases, Quotas, ServiceIdentity};
    use crate::domain::value_objects::project::ProjectName;
    use crate::infrastructure::CedarPermissionService;
    use crate::infrastructure::persistence::postgres::{
        PgAuthzEntityProvider, PgDefaultRoleBindingRepository, PgGrantRepository,
        PgPolicyRepository, PgRoleRepository, PgUserProjectRepository, PgUserRepository,
    };
    use std::sync::Arc;

    let org = seed_org(&pool, "limited").await;
    let permission = Arc::new(
        CedarPermissionService::new(
            Arc::new(PgAuthzEntityProvider::new(pool.clone())),
            Arc::new(PgRoleRepository::new(pool.clone())),
            Arc::new(PgGrantRepository::new(pool.clone())),
            Arc::new(PgPolicyRepository::new(pool.clone())),
            Arc::new(NoopAuditLog),
        )
        .await
        .expect("cedar"),
    );
    let uc = ProjectUseCases::new(
        Arc::new(PgProjectRepository::new(pool.clone())),
        Arc::new(PgUserProjectRepository::new(pool.clone())),
        Arc::new(PgUserRepository::new(pool.clone())),
        Arc::new(PgGrantRepository::new(pool.clone())),
        permission.clone(),
        permission,
        Arc::new(PgDefaultRoleBindingRepository::new(pool.clone())),
        Quotas {
            max_projects_per_org: 2,
        },
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
        repo.list_by_organization(org_a.id(), None)
            .await
            .unwrap()
            .metadata()
            .total_count(),
        2,
    );
    assert_eq!(
        repo.list_by_organization(org_b.id(), None)
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
/// the membership row AND a `project-admin` owner grant — so a project is never
/// left without an administrator.
#[sqlx::test(migrations = "../../migrations")]
async fn provision_with_owner_writes_membership_and_owner_grant(pool: PgPool) {
    use crate::application::UserProjectRepository;
    use crate::application::authz::grant::{
        Grant, GrantRepository, GrantTarget, PROJECT_ADMIN_ROLE, Principal, Scope,
    };
    use crate::domain::value_objects::role::RoleName;
    use crate::infrastructure::persistence::postgres::{
        PgGrantRepository, PgUserProjectRepository,
    };

    let org = seed_org(&pool, "acme").await;
    let owner = seed_user(&pool, "alice").await;
    let project = project(&org, "rocket");
    let grant = Grant::new(
        Principal::User(owner.id().clone()),
        RoleName::new(PROJECT_ADMIN_ROLE).unwrap(),
        Scope::Project(project.id().clone()),
    );

    let repo = PgProjectRepository::new(pool.clone());
    repo.provision_with_owner(&project, owner.id(), &grant)
        .await
        .expect("provision");

    assert_eq!(
        repo.find_by_id(project.id()).await.unwrap().id(),
        project.id()
    );
    assert!(
        PgUserProjectRepository::new(pool.clone())
            .is_member(owner.id(), project.id())
            .await
            .unwrap(),
        "creator should be a project member",
    );
    let grants = PgGrantRepository::new(pool).list_all().await.unwrap();
    assert!(
        grants.iter().any(|g| {
            matches!(&g.principal, Principal::User(u) if u.as_str() == owner.id().as_str())
                && matches!(&g.target, GrantTarget::Role(r) if r.as_str() == PROJECT_ADMIN_ROLE)
                && matches!(&g.scope, Scope::Project(p) if p.as_str() == project.id().as_str())
        }),
        "creator should hold a project-admin owner grant",
    );
}

/// The provisioning transaction is atomic: a failure on any insert (here a
/// dangling owner id → FK violation) rolls back the project too.
#[sqlx::test(migrations = "../../migrations")]
async fn provision_with_owner_rolls_back_on_failure(pool: PgPool) {
    use crate::application::authz::grant::{Grant, PROJECT_ADMIN_ROLE, Principal, Scope};
    use crate::domain::entities::UserId;
    use crate::domain::value_objects::role::RoleName;

    let org = seed_org(&pool, "acme").await;
    let project = project(&org, "rocket");
    let ghost = UserId::new("does-not-exist");
    let grant = Grant::new(
        Principal::User(ghost.clone()),
        RoleName::new(PROJECT_ADMIN_ROLE).unwrap(),
        Scope::Project(project.id().clone()),
    );

    let repo = PgProjectRepository::new(pool.clone());
    let err = repo
        .provision_with_owner(&project, &ghost, &grant)
        .await
        .expect_err("FK violation on the dangling owner");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "FK violation maps to Conflict"
    );
    assert!(
        matches!(
            repo.find_by_id(project.id()).await,
            Err(DomainError::NotFound { .. })
        ),
        "the project insert must have rolled back",
    );
}
