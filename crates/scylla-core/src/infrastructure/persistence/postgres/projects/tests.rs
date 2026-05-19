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
