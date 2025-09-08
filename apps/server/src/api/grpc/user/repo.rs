use crate::api::base::BaseRepository;
use crate::api::base::diesel_repo_base::Repository;
use crate::api::grpc::user::UserRepository;
use crate::api::grpc::user::dto::{NewUser, UpdateUser};
use crate::api::grpc::user::models::User;
use crate::database::DieselPool;
use anyhow::Context;
use async_trait::async_trait;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::SelectableHelper;
use diesel::{ExpressionMethods, OptionalExtension};
use repository_derive::DieselRepository;

#[derive(DieselRepository)]
pub struct UserRepositoryDiesel {
    base: BaseRepository,
}

#[async_trait]
impl UserRepository for UserRepositoryDiesel {
    async fn create_user(&self, new_user: NewUser) -> anyhow::Result<User> {
        use crate::database::schema::users::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        let partial: User = diesel::insert_into(users)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .context("Failed to create user")?;

        Ok(partial)
    }

    /// Fetches a user by their UUID.
    /// # Arguments
    /// * `user_uuid` - The UUID of the user to be fetched.
    /// # Returns
    /// * `Result<Option<User>>` - The user if found, or None if not found, or an error if the operation fails.
    async fn get_user_by_uuid(&self, req_user_uuid: uuid::Uuid) -> anyhow::Result<Option<User>> {
        use crate::database::schema::users::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        let user = users
            .filter(id.eq(req_user_uuid))
            .first::<User>(&mut conn)
            .optional()
            .context("Failed to fetch user by UUID")?;

        Ok(user)
    }

    async fn list_users(&self, limit: i64, offset: i64) -> anyhow::Result<Vec<User>> {
        use crate::database::schema::users::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        let rows: Vec<User> = users
            .select(User::as_select())
            .order(created_at.desc())
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .context("Failed to list users")?;

        Ok(rows)
    }

    async fn update_user(
        &self,
        req_user_uuid: uuid::Uuid,
        changes: UpdateUser,
    ) -> anyhow::Result<Option<User>> {
        use crate::database::schema::users::dsl::*;
        // Optionnel: empêcher un no-op si rien n’est fourni (hors updated_at)
        let nothing_to_update = changes.username.is_none() && changes.password_hash.is_none();
        if nothing_to_update {
            return Ok(None);
        }

        let mut conn = Repository::get_connection(self)?;

        let target = users.filter(id.eq(req_user_uuid));

        let row = diesel::update(target)
            .set(&changes)
            .get_result::<User>(&mut conn)
            .optional()
            .context("Failed to update user")?;

        Ok(row)
    }

    async fn deactivate_user(&self, req_user_uuid: uuid::Uuid) -> anyhow::Result<Option<User>> {
        use crate::database::schema::users::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        let target = users.filter(id.eq(req_user_uuid));

        let row = diesel::update(target)
            .set(is_active.eq(false))
            .get_result::<User>(&mut conn)
            .optional()
            .context("Failed to deactivate user")?;

        Ok(row)
    }

    async fn get_user_by_username(&self, req_username: String) -> anyhow::Result<Option<User>> {
        use crate::database::schema::users::dsl::*;
        let mut conn = Repository::get_connection(self)?;

        let user = users
            .filter(username.eq(req_username))
            .first::<User>(&mut conn)
            .optional()
            .context("Failed to fetch user by username")?;

        Ok(user)
    }
}
