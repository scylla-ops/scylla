use crate::api::grpc::project::models::{
    InsertableProject, Project, ProjectPatch, UserProjectRelation,
};
use crate::api::grpc::project::repos::ProjectRepository;
use crate::api::grpc::tables;
use crate::api::grpc::user::models::User;
use crate::database::db;
use anyhow::Context;
use async_trait::async_trait;
use protocol::serde_json;
use serde::Deserialize;
use surrealdb::{RecordId, RecordIdKey};

pub struct ProjectRepositorySurreal;

#[async_trait]
impl ProjectRepository for ProjectRepositorySurreal {
    async fn create_project(new_project: InsertableProject) -> anyhow::Result<Project> {
        let rec: Option<Project> = db()
            .create(tables::projects::TABLE)
            .content(new_project)
            .await
            .context("Failed to create project")?;

        let row = rec.context("Failed to fetch project")?;
        Ok(row)
    }

    async fn get_project_by_id(project_id: RecordIdKey) -> anyhow::Result<Option<Project>> {
        let rec: Option<Project> = db().select((tables::projects::TABLE, project_id)).await?;
        Ok(rec)
    }

    async fn get_project_by_name_and_org(
        name: String,
        org_id: RecordIdKey,
    ) -> anyhow::Result<Option<Project>> {
        let query = format!(
            "SELECT * FROM {} WHERE name = $name AND organization = $org LIMIT 1",
            tables::projects::TABLE
        );
        let mut result = db()
            .query(query)
            .bind(("name", name))
            .bind(("org", tables::organizations::to_record_id(org_id)))
            .await?;
        let recs: Vec<Project> = result.take(0)?;
        Ok(recs.into_iter().next())
    }

    async fn list_projects(limit: i64, offset: i64) -> anyhow::Result<Vec<Project>> {
        let query = format!(
            "SELECT * FROM {} ORDER BY created_at DESC LIMIT $limit START $offset",
            tables::projects::TABLE
        );
        let mut result = db()
            .query(query)
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;
        let recs: Vec<Project> = result.take(0)?;
        Ok(recs.into_iter().collect())
    }

    async fn list_organization_projects(
        org_id: RecordIdKey,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<Project>> {
        let query = format!(
            "SELECT * FROM {} WHERE organization = $org ORDER BY created_at DESC LIMIT $limit START $offset",
            tables::projects::TABLE
        );
        let mut result = db()
            .query(query)
            .bind(("org", tables::organizations::to_record_id(org_id)))
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;
        let recs: Vec<Project> = result.take(0)?;
        Ok(recs.into_iter().collect())
    }

    async fn update_project(
        project_id: RecordIdKey,
        changes: ProjectPatch,
    ) -> anyhow::Result<Option<Project>> {
        let rec: Option<Project> = db()
            .update((tables::projects::TABLE, project_id))
            .merge(changes)
            .await?;
        Ok(rec)
    }

    async fn deactivate_project(project_id: RecordIdKey) -> anyhow::Result<Option<Project>> {
        let rec: Option<Project> = db()
            .update((tables::projects::TABLE, project_id))
            .merge(serde_json::json!({
                "is_active": false,
            }))
            .await?;
        Ok(rec)
    }

    async fn add_user_to_project(
        user_id: RecordIdKey,
        project_id: RecordIdKey,
        role: String,
    ) -> anyhow::Result<UserProjectRelation> {
        let query = format!(
            "RELATE $user->{}->$project SET role = $role",
            tables::user_project::TABLE
        );
        let mut result = db()
            .query(query)
            .bind(("user", tables::users::to_record_id(user_id)))
            .bind(("project", tables::projects::to_record_id(project_id)))
            .bind(("role", role))
            .await?;
        let relation: Option<UserProjectRelation> = result.take(0)?;
        relation.ok_or_else(|| anyhow::anyhow!("Failed to create user-project relation"))
    }

    async fn remove_user_from_project(
        user_id: RecordIdKey,
        project_id: RecordIdKey,
    ) -> anyhow::Result<()> {
        let query = format!(
            "DELETE $user->{} WHERE out = $project",
            tables::user_project::TABLE
        );
        db().query(query)
            .bind(("user", tables::users::to_record_id(user_id)))
            .bind(("project", tables::projects::to_record_id(project_id)))
            .await?;
        Ok(())
    }

    async fn list_project_users(
        project_id: RecordIdKey,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<(User, UserProjectRelation)>> {
        let query = format!(
            "SELECT in, role, joined_at, id FROM {} WHERE out = $project ORDER BY joined_at DESC LIMIT $limit START $offset FETCH in",
            tables::user_project::TABLE
        );
        let mut result = db()
            .query(query)
            .bind(("project", tables::projects::to_record_id_ref(&project_id)))
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;

        #[derive(Deserialize)]
        struct QueryRow {
            #[serde(rename = "in")]
            user: User,
            role: String,
            joined_at: chrono::DateTime<chrono::Utc>,
            id: RecordId,
        }

        let data: Vec<QueryRow> = result.take(0)?;

        let project_record_id = tables::projects::to_record_id(project_id);

        Ok(data
            .into_iter()
            .map(|row| {
                let relation = UserProjectRelation {
                    id: row.id,
                    user: row.user.id.clone(),
                    project: project_record_id.clone(),
                    role: row.role,
                    joined_at: row.joined_at,
                };
                (row.user, relation)
            })
            .collect())
    }

    async fn list_user_projects(
        user_id: RecordIdKey,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<(Project, UserProjectRelation)>> {
        let query = format!(
            "SELECT out, role, joined_at, id FROM {} WHERE in = $user ORDER BY joined_at DESC LIMIT $limit START $offset FETCH out",
            tables::user_project::TABLE
        );
        let mut result = db()
            .query(query)
            .bind(("user", tables::users::to_record_id_ref(&user_id)))
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await?;

        #[derive(Deserialize)]
        struct QueryRow {
            #[serde(rename = "out")]
            project: Project,
            role: String,
            joined_at: chrono::DateTime<chrono::Utc>,
            id: RecordId,
        }

        let data: Vec<QueryRow> = result.take(0)?;

        let user_record_id = tables::users::to_record_id(user_id);

        Ok(data
            .into_iter()
            .map(|row| {
                let relation = UserProjectRelation {
                    id: row.id,
                    user: user_record_id.clone(),
                    project: row.project.id.clone(),
                    role: row.role,
                    joined_at: row.joined_at,
                };
                (row.project, relation)
            })
            .collect())
    }
}

