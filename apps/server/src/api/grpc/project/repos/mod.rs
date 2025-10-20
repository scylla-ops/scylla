use crate::api::grpc::project::models::{
    InsertableProject, Project, ProjectPatch, UserProjectRelation,
};
use crate::api::grpc::user::models::User;
use async_trait::async_trait;
use surrealdb::RecordIdKey;

#[cfg(feature = "surreal")]
pub mod surreal;

#[async_trait]
pub trait ProjectRepository: Send + Sync + 'static {
    async fn create_project(new_project: InsertableProject) -> anyhow::Result<Project>;
    async fn get_project_by_id(
        #[cfg(feature = "surreal")] project_id: RecordIdKey,
    ) -> anyhow::Result<Option<Project>>;
    async fn get_project_by_name_and_org(
        name: String,
        #[cfg(feature = "surreal")] org_id: RecordIdKey,
    ) -> anyhow::Result<Option<Project>>;
    async fn list_projects(limit: i64, offset: i64) -> anyhow::Result<Vec<Project>>;
    async fn list_organization_projects(
        #[cfg(feature = "surreal")] org_id: RecordIdKey,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<Project>>;
    async fn update_project(
        #[cfg(feature = "surreal")] project_id: RecordIdKey,
        changes: ProjectPatch,
    ) -> anyhow::Result<Option<Project>>;
    async fn deactivate_project(
        #[cfg(feature = "surreal")] project_id: RecordIdKey,
    ) -> anyhow::Result<Option<Project>>;

    async fn add_user_to_project(
        #[cfg(feature = "surreal")] user_id: RecordIdKey,
        #[cfg(feature = "surreal")] project_id: RecordIdKey,
        role: String,
    ) -> anyhow::Result<UserProjectRelation>;
    async fn remove_user_from_project(
        #[cfg(feature = "surreal")] user_id: RecordIdKey,
        #[cfg(feature = "surreal")] project_id: RecordIdKey,
    ) -> anyhow::Result<()>;
    async fn list_project_users(
        #[cfg(feature = "surreal")] project_id: RecordIdKey,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<(User, UserProjectRelation)>>;
    async fn list_user_projects(
        #[cfg(feature = "surreal")] user_id: RecordIdKey,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<(Project, UserProjectRelation)>>;
}
