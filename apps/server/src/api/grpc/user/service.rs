use crate::api::grpc::user::models::{
    CreateUserInput, InsertableUser, UpdateUserInput, User, UserPatch,
};
use crate::api::grpc::user::password::ScyllaPassword;
use crate::api::grpc::user::repos::UserRepository;
use crate::api::grpc::user::username::{ScyllaUsername, UsernameError};
use crate::api::grpc::utils::{Id, hash_password};
use derive_more::Constructor;
use thiserror::Error;
use tracing::error;

#[derive(Constructor)]
pub struct UserService<R: UserRepository> {
    _marker: std::marker::PhantomData<R>,
}

#[derive(Debug, Error)]
pub enum UserDomainError {
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Invalid username: {0}")]
    InvalidUsername(#[from] UsernameError),
    #[error("Invalid password: {0}")]
    InvalidPassword(String),
    #[error("Invalid pagination parameter: {field}")]
    InvalidPagination { field: &'static str },
    #[error("User not found")]
    UserNotFound,
    #[error("Hashing error: {0}")]
    Hashing(#[from] argon2::Error),
    #[error(transparent)]
    Repo(#[from] anyhow::Error),
}

impl<R: UserRepository> UserService<R> {
    pub async fn create_user(req: CreateUserInput) -> Result<User, UserDomainError> {
        let CreateUserInput {
            username: username_str,
            password: password_str,
        } = req;
        let username = ScyllaUsername::new(username_str)?;
        let password = ScyllaPassword::new(password_str)
            .map_err(|e| UserDomainError::InvalidPassword(e.to_string()))?;

        let new_user = InsertableUser {
            username,
            password_hash: hash_password(password.as_str())?,
        };

        R::create_user(new_user)
            .await
            .map_err(UserDomainError::Repo)
    }

    pub async fn get_user(user_id: Id) -> Result<User, UserDomainError> {
        let opt = R::get_user_by_id(user_id).await?;
        match opt {
            Some(u) => Ok(u),
            None => Err(UserDomainError::UserNotFound),
        }
    }

    pub async fn list_users(
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<User>, usize), UserDomainError> {
        const MAX_PAGE_SIZE: u32 = 100;
        if page == 0 {
            return Err(UserDomainError::InvalidPagination { field: "page" });
        }
        if page_size == 0 {
            return Err(UserDomainError::InvalidPagination { field: "page_size" });
        }
        if page_size > MAX_PAGE_SIZE {
            return Err(UserDomainError::Validation(format!(
                "page_size must be <= {}",
                MAX_PAGE_SIZE
            )));
        }
        let limit_i64: i64 = page_size.into();
        let offset_u128 = (page as u128)
            .saturating_sub(1)
            .saturating_mul(page_size as u128);
        let offset_i64 = i64::try_from(offset_u128)
            .map_err(|_| UserDomainError::Validation("computed offset is too big".into()))?;
        let list = R::list_users(limit_i64, offset_i64).await?;
        let total = list.len();
        Ok((list, total))
    }

    pub async fn update_user(user_id: Id, req: UpdateUserInput) -> Result<User, UserDomainError> {
        let UpdateUserInput {
            username,
            password,
            is_active,
        } = req;

        let username = match username {
            Some(u) => Some(ScyllaUsername::new(u)?),
            None => None,
        };

        let password_hash = match password {
            Some(p) => {
                let pwd = ScyllaPassword::new(p)
                    .map_err(|e| UserDomainError::InvalidPassword(e.to_string()))?;
                Some(hash_password(pwd.as_str())?)
            }
            None => None,
        };

        let update_user = UserPatch {
            username,
            password_hash,
            is_active,
        };

        let opt = R::update_user(user_id, update_user).await?;

        match opt {
            Some(u) => Ok(u),
            None => Err(UserDomainError::UserNotFound),
        }
    }

    pub async fn deactivate_user(user_id: Id) -> Result<(), UserDomainError> {
        let res = R::deactivate_user(user_id).await?;
        match res {
            Some(_) => Ok(()),
            None => Err(UserDomainError::UserNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api::grpc::user::models::CreateUserInput;
    use crate::api::grpc::user::repos::surreal::UserRepositorySurreal;
    use crate::api::grpc::user::service::UserService;
    use crate::api::grpc::user::username::ScyllaUsername;
    use crate::database::{DB, apply_migrations};
    use std::sync::Arc;
    use surrealdb::Surreal;
    use surrealdb::engine::any::Any;

    pub async fn setup_test_db() {
        DB.get_or_init(|| async {
            let client: Arc<Surreal<Any>> =
                Arc::from(surrealdb::engine::any::connect("mem://").await.unwrap());
            client.use_ns("test").use_db("user").await.unwrap();
            apply_migrations(client.clone()).await;
            client
        })
        .await;
    }

    #[tokio::test]
    async fn create_valid_user() {
        setup_test_db().await;

        static USERNAME: &str = "user1";
        static PASSWORD: &str = "password123";

        let create_input = CreateUserInput {
            username: USERNAME.to_owned(),
            password: PASSWORD.to_owned(),
        };

        let created_user = UserService::<UserRepositorySurreal>::create_user(create_input).await;
        assert!(created_user.is_ok());

        let user_id = created_user.unwrap().id.key().to_string();
        let fetched_user = UserService::<UserRepositorySurreal>::get_user(user_id.clone()).await;

        assert!(fetched_user.is_ok());
        let fetched_user = fetched_user.unwrap();
        assert_eq!(fetched_user.id.key().to_string(), user_id);
        assert_eq!(
            fetched_user.username,
            ScyllaUsername::new(USERNAME.to_owned()).unwrap()
        );
        assert!(!fetched_user.is_active);
    }
}
