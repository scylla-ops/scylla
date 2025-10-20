use crate::api::grpc::organization::models::{
    InsertableOrganization, Organization, OrganizationPatch, UserOrganizationRelation,
};
use crate::api::grpc::user::models::User;
use async_trait::async_trait;
use surrealdb::RecordIdKey;

#[cfg(feature = "surreal")]
pub mod surreal;

#[async_trait]
pub trait OrganizationRepository: Send + Sync + 'static {
    async fn create_organization(new_org: InsertableOrganization) -> anyhow::Result<Organization>;
    async fn get_organization_by_id(
        #[cfg(feature = "surreal")] org_id: RecordIdKey,
    ) -> anyhow::Result<Option<Organization>>;
    async fn get_organization_by_name(name: String) -> anyhow::Result<Option<Organization>>;
    async fn list_organizations(limit: i64, offset: i64) -> anyhow::Result<Vec<Organization>>;
    async fn update_organization(
        #[cfg(feature = "surreal")] org_id: RecordIdKey,
        changes: OrganizationPatch,
    ) -> anyhow::Result<Option<Organization>>;
    async fn deactivate_organization(
        #[cfg(feature = "surreal")] org_id: RecordIdKey,
    ) -> anyhow::Result<Option<Organization>>;

    async fn add_user_to_organization(
        #[cfg(feature = "surreal")] user_id: RecordIdKey,
        #[cfg(feature = "surreal")] org_id: RecordIdKey,
        role: String,
    ) -> anyhow::Result<UserOrganizationRelation>;
    async fn remove_user_from_organization(
        #[cfg(feature = "surreal")] user_id: RecordIdKey,
        #[cfg(feature = "surreal")] org_id: RecordIdKey,
    ) -> anyhow::Result<()>;
    async fn list_organization_users(
        #[cfg(feature = "surreal")] org_id: RecordIdKey,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<(User, UserOrganizationRelation)>>;
    async fn list_user_organizations(
        #[cfg(feature = "surreal")] user_id: RecordIdKey,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<(Organization, UserOrganizationRelation)>>;
}
