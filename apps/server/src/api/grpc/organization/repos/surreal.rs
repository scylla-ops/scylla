use crate::api::grpc::organization::models::{
    InsertableOrganization, Organization, OrganizationPatch, UserOrganizationRelation,
};
use crate::api::grpc::organization::repos::OrganizationRepository;
use crate::api::grpc::tables;
use crate::api::grpc::user::models::User;
use crate::database::db;
use anyhow::Context;
use async_trait::async_trait;
use protocol::serde_json;
use serde::Deserialize;
use surrealdb::RecordIdKey;

pub struct OrganizationRepositorySurreal;

#[async_trait]
impl OrganizationRepository for OrganizationRepositorySurreal {
    async fn create_organization(new_org: InsertableOrganization) -> anyhow::Result<Organization> {
        let rec: Option<Organization> = db()
            .create(tables::organizations::TABLE)
            .content(new_org)
            .await
            .context("Failed to create organization")?;

        let row = rec.context("Failed to fetch organization")?;
        Ok(row)
    }

    async fn get_organization_by_id(org_id: RecordIdKey) -> anyhow::Result<Option<Organization>> {
        let rec: Option<Organization> = db().select((tables::organizations::TABLE, org_id)).await?;
        Ok(rec)
    }

    async fn get_organization_by_name(name: String) -> anyhow::Result<Option<Organization>> {
        let rec: Option<Organization> = db().select((tables::organizations::TABLE, name)).await?;
        Ok(rec)
    }

    async fn list_organizations(limit: i64, offset: i64) -> anyhow::Result<Vec<Organization>> {
        let query = format!(
            "SELECT * FROM {} ORDER BY created_at DESC LIMIT $limit START $offset",
            tables::organizations::TABLE
        );
        let mut result = db()
            .query(query)
            .bind(("limit", limit.clone()))
            .bind(("offset", offset.clone()))
            .await?;
        let recs: Vec<Organization> = result.take(0)?;
        Ok(recs.into_iter().collect())
    }

    async fn update_organization(
        org_id: RecordIdKey,
        changes: OrganizationPatch,
    ) -> anyhow::Result<Option<Organization>> {
        let rec: Option<Organization> = db()
            .update((tables::organizations::TABLE, org_id))
            .merge(changes)
            .await?;
        Ok(rec)
    }

    async fn deactivate_organization(org_id: RecordIdKey) -> anyhow::Result<Option<Organization>> {
        let rec: Option<Organization> = db()
            .update((tables::organizations::TABLE, org_id))
            .merge(serde_json::json!({
                "is_active": false,
            }))
            .await?;
        Ok(rec)
    }

    async fn add_user_to_organization(
        user_id: RecordIdKey,
        org_id: RecordIdKey,
        role: String,
    ) -> anyhow::Result<UserOrganizationRelation> {
        let query = format!(
            "RELATE $user->{}->$org SET role = $role",
            tables::user_organization::TABLE
        );
        let mut result = db()
            .query(query)
            .bind(("user", tables::users::to_record_id(user_id)))
            .bind(("org", tables::organizations::to_record_id(org_id)))
            .bind(("role", role))
            .await?;
        let relation: Option<UserOrganizationRelation> = result.take(0)?;
        relation.ok_or_else(|| anyhow::anyhow!("Failed to create user-organization relation"))
    }

    async fn remove_user_from_organization(
        user_id: RecordIdKey,
        org_id: RecordIdKey,
    ) -> anyhow::Result<()> {
        let query = format!(
            "DELETE $user->{} WHERE out = $org",
            tables::user_organization::TABLE
        );
        db().query(query)
            .bind(("user", tables::users::to_record_id(user_id)))
            .bind(("org", tables::organizations::to_record_id(org_id)))
            .await?;
        Ok(())
    }

    async fn list_organization_users(
        org_id: RecordIdKey,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<(User, UserOrganizationRelation)>> {
        let query = format!(
            "SELECT VALUE [in, role, joined_at, id] FROM {} WHERE out = $org ORDER BY joined_at DESC LIMIT $limit START $offset",
            tables::user_organization::TABLE
        );
        let mut result = db()
            .query(query)
            .bind(("org", tables::organizations::to_record_id_ref(&org_id)))
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;

        #[derive(Deserialize)]
        struct QueryRow(User, String, chrono::DateTime<chrono::Utc>, RecordIdKey);

        let data: Vec<QueryRow> = result.take(0)?;

        let org_record_id = tables::organizations::to_record_id(org_id);

        Ok(data
            .into_iter()
            .map(|QueryRow(user, role, joined_at, relation_id)| {
                let relation = UserOrganizationRelation {
                    id: tables::user_organization::to_record_id(relation_id),
                    user: user.id.clone(),
                    organization: org_record_id.clone(),
                    role,
                    joined_at,
                };
                (user, relation)
            })
            .collect())
    }

    async fn list_user_organizations(
        user_id: RecordIdKey,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<(Organization, UserOrganizationRelation)>> {
        let query = format!(
            "SELECT VALUE [out, role, joined_at, id] FROM {} WHERE in = $user ORDER BY joined_at DESC LIMIT $limit START $offset",
            tables::user_organization::TABLE
        );
        let mut result = db()
            .query(query)
            .bind(("user", tables::users::to_record_id_ref(&user_id)))
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;

        #[derive(Deserialize)]
        struct QueryRow(
            Organization,
            String,
            chrono::DateTime<chrono::Utc>,
            RecordIdKey,
        );

        let data: Vec<QueryRow> = result.take(0)?;

        let user_record_id = tables::users::to_record_id(user_id);

        Ok(data
            .into_iter()
            .map(|QueryRow(org, role, joined_at, relation_id)| {
                let relation = UserOrganizationRelation {
                    id: tables::user_organization::to_record_id(relation_id),
                    user: user_record_id.clone(),
                    organization: org.id.clone(),
                    role,
                    joined_at,
                };
                (org, relation)
            })
            .collect())
    }
}
