use crate::api::v1::common::base::{BaseRepository, Repository};
use crate::api::v1::models::users::User;
use crate::api::v1::modules::user::dto::NewUser;
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
}

// Implement Repository trait for CommandRepository by delegating to the base repository
impl Repository for UserRepository {
    fn get_pool(&self) -> &DieselPool {
        self.base.get_pool()
    }
}
