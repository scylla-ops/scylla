use crate::api::grpc::user::models::{InsertableUser, User, UserPatch};
use crate::api::grpc::user::repos::{TABLE, UserRepository};
use crate::api::grpc::utils::Id;
use crate::database::db;
use anyhow::Context;
use async_trait::async_trait;
use protocol::serde_json;
use tracing::debug;

pub struct UserRepositorySurreal;

#[async_trait]
impl UserRepository for UserRepositorySurreal {
    async fn create_user(new_user: InsertableUser) -> anyhow::Result<User> {
        let rec: Option<User> =
            dbg!(db().create(TABLE).content(new_user).await).context("Failed to create user")?;

        let row = rec.context("Failed to fetch user")?;
        Ok(row)
    }

    async fn get_user_by_id(user_id: Id) -> anyhow::Result<Option<User>> {
        let rec: Option<User> = db().select((TABLE, user_id)).await?;
        Ok(rec)
    }

    async fn get_user_by_username(username: String) -> anyhow::Result<Option<User>> {
        let query = format!("SELECT * FROM {} WHERE username = $username", TABLE);
        debug!(query = ?query, username = ?username);
        let mut response = db().query(query).bind(("username", username)).await?;
        let recs: Vec<User> = response.take(0)?;
        Ok(recs.into_iter().next())
    }

    async fn list_users(limit: i64, offset: i64) -> anyhow::Result<Vec<User>> {
        let query = format!(
            "SELECT * FROM {} ORDER BY created_at DESC LIMIT {} START {}",
            TABLE, limit, offset
        );
        let mut recs = db().query(query).await?;
        let recs = recs.take(0)?;
        Ok(recs)
    }

    async fn update_user(user_id: Id, changes: UserPatch) -> anyhow::Result<Option<User>> {
        let rec: Option<User> = db().update((TABLE, user_id)).merge(changes).await?;

        Ok(rec)
    }

    async fn deactivate_user(user_id: Id) -> anyhow::Result<Option<User>> {
        let rec: Option<User> = db()
            .update((TABLE, user_id))
            .merge(serde_json::json!({
                "is_active": false,
            }))
            .await?;
        Ok(rec)
    }
}
