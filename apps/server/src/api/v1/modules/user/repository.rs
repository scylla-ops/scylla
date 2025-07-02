use crate::api::v1::common::base::{BaseRepository, Repository};
use crate::api::v1::models::users::User;
use crate::api::v1::modules::user::dto::{NewUser, UpdateUser};
use crate::database::DieselPool;
use anyhow::{Context, Result};
use diesel::prelude::*;
use tracing::debug;

// Example command repository
pub struct UserRepository {
    base: BaseRepository,
}

impl UserRepository {
    pub fn new(pool: DieselPool) -> Self {
        Self {
            base: BaseRepository::new(pool),
        }
    }

    /// Creates a new user in the database.
    /// # Arguments
    /// * `new_user` - The new user data to be inserted.
    /// # Returns
    /// * `Result<usize>` - The number of rows inserted, or an error if the operation fails.
    pub async fn create_user(&self, new_user: NewUser) -> Result<usize> {
        use crate::database::schema::users::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        let inserted_count = diesel::insert_into(users)
            .values(&new_user)
            .execute(&mut conn)
            .context("Failed to insert new user")?;

        debug!("Inserted {} new user(s)", inserted_count);
        Ok(inserted_count)
    }

    /// Fetches a user by their UUID.
    /// # Arguments
    /// * `user_uuid` - The UUID of the user to be fetched.
    /// # Returns
    /// * `Result<Option<User>>` - The user if found, or None if not found, or an error if the operation fails.
    pub async fn get_user_by_uuid(&self, user_uuid: uuid::Uuid) -> Result<Option<User>> {
        use crate::database::schema::users::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        let user = users
            .filter(id.eq(user_uuid))
            .first::<User>(&mut conn)
            .optional()
            .context("Failed to fetch user by UUID")?;

        debug!("Fetched user: {:?}", user);
        Ok(user)
    }

    /// Fetches all users from the database.
    /// # Returns
    /// * `Result<Vec<User>>` - A vector of users, or an error if the operation fails.
    pub async fn get_all_users(&self) -> Result<Vec<User>> {
        use crate::database::schema::users::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        let users_list = users
            .load::<User>(&mut conn)
            .context("Failed to fetch all users")?;

        debug!("Fetched {} users", users_list.len());
        Ok(users_list)
    }

    /// Updates a user by their UUID.
    /// # Arguments
    /// * `user_uuid` - The UUID of the user to be updated.
    /// * `updated_user` - The updated user data.
    /// # Returns
    /// * `Result<usize>` - The number of rows updated, or an error if the operation fails.
    pub async fn update_user_by_uuid(
        &self,
        user_uuid: uuid::Uuid,
        updated_user: UpdateUser,
    ) -> Result<usize> {
        use crate::database::schema::users::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        let updated_count = diesel::update(users.filter(id.eq(user_uuid)))
            .set(updated_user)
            .execute(&mut conn)
            .context("Failed to update user by UUID")?;

        debug!("Updated {} user(s)", updated_count);
        Ok(updated_count)
    }

    /// Deactivates a user by their UUID.
    /// # Arguments
    /// * `user_uuid` - The UUID of the user to be deactivated.
    /// # Returns
    /// * `Result<usize>` - The number of rows updated, or an error if the operation fails.
    pub async fn deactivate_user_by_uuid(&self, user_uuid: uuid::Uuid) -> Result<usize> {
        use crate::database::schema::users::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        let updated_count = diesel::update(users.filter(id.eq(user_uuid)))
            .set(is_active.eq(false))
            .execute(&mut conn)
            .context("Failed to deactivate user by UUID")?;

        debug!("Deactivated {} user(s)", updated_count);
        Ok(updated_count)
    }
}

// Implement Repository trait for CommandRepository by delegating to the base repository
impl Repository for UserRepository {
    fn get_pool(&self) -> &DieselPool {
        self.base.get_pool()
    }
}
