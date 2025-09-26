use crate::api::grpc::user::dto::{NewUser, NewUserRequest, UpdateUser, UserFields};
use crate::api::grpc::user::repo::UserRepositoryDiesel;
use crate::api::grpc::user::{UserRepository, dto};
use crate::database::get_existing_db;
use bcrypt::BcryptError;
use chrono::Utc;
use derive_more::Constructor;
use std::sync::{Arc, LazyLock};
use thiserror::Error;
use tracing::error;
use uuid::Uuid;
use validator::Validate;

#[derive(Constructor)]
pub struct UserService {
    repo: Arc<dyn UserRepository>,
}

pub static USER_SERVICE: LazyLock<Arc<UserService>> = LazyLock::new(|| {
    let diesel_db = get_existing_db();

    Arc::new(UserService::new(Arc::new(UserRepositoryDiesel::new(
        diesel_db.clone(),
    ))))
});

#[derive(Debug, Error)]
pub enum UserDomainError {
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("User not found")]
    UserNotFound,
    #[error("Hashing error: {0}")]
    Hashing(#[from] BcryptError),
    #[error(transparent)]
    Repo(#[from] anyhow::Error),
}

impl UserService {
    fn hash_password(password: &str) -> Result<String, BcryptError> {
        //todo: ne pas utiliser le coût par défaut en production
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
    }

    pub async fn create_user(
        &self,
        req: NewUserRequest,
    ) -> Result<crate::api::grpc::user::models::User, UserDomainError> {
        if let Err(e) = req.validate() {
            return Err(UserDomainError::Validation(e.to_string()));
        }
        let password_hash = Self::hash_password(&req.fields.password.clone().unwrap())?;
        let new_user = NewUser {
            username: req.fields.username.clone().unwrap(),
            password_hash,
        };
        let user = self
            .repo
            .create_user(new_user)
            .await
            .map_err(UserDomainError::Repo)?;
        Ok(user)
    }

    pub async fn get_user(
        &self,
        user_uuid: Uuid,
    ) -> Result<crate::api::grpc::user::models::User, UserDomainError> {
        let opt = self
            .repo
            .get_user_by_uuid(user_uuid)
            .await
            .map_err(UserDomainError::Repo)?;
        match opt {
            Some(u) => Ok(u),
            None => Err(UserDomainError::UserNotFound),
        }
    }

    pub async fn list_users(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<crate::api::grpc::user::models::User>, usize), UserDomainError> {
        if page == 0 {
            return Err(UserDomainError::Validation("page must be >= 1".into()));
        }
        if page_size == 0 {
            return Err(UserDomainError::Validation("page_size must be >= 1".into()));
        }
        let limit_i64 = i64::try_from(page_size)
            .map_err(|_| UserDomainError::Validation("page_size is too big".into()))?;
        let offset_u128 = (page as u128)
            .saturating_sub(1)
            .saturating_mul(page_size as u128);
        let offset_i64 = i64::try_from(offset_u128)
            .map_err(|_| UserDomainError::Validation("computed offset is too big".into()))?;
        let list = self
            .repo
            .list_users(limit_i64, offset_i64)
            .await
            .map_err(UserDomainError::Repo)?;
        let total = list.len();
        Ok((list, total))
    }

    pub async fn update_user(
        &self,
        user_uuid: Uuid,
        req: dto::UpdateUserRequest,
    ) -> Result<crate::api::grpc::user::models::User, UserDomainError> {
        if let Err(e) = req.validate() {
            return Err(UserDomainError::Validation(e.to_string()));
        }
        let UserFields {
            username, password, ..
        } = req.fields;
        let password_hash = if let Some(ref pwd) = password {
            Some(Self::hash_password(pwd)?)
        } else {
            None
        };
        let update_user = UpdateUser {
            username,
            password_hash,
            updated_at: Utc::now().naive_utc(),
        };
        let opt = self
            .repo
            .update_user(user_uuid, update_user)
            .await
            .map_err(UserDomainError::Repo)?;
        match opt {
            Some(u) => Ok(u),
            None => Err(UserDomainError::UserNotFound),
        }
    }

    pub async fn deactivate_user(&self, user_uuid: Uuid) -> Result<(), UserDomainError> {
        let res = self
            .repo
            .deactivate_user(user_uuid)
            .await
            .map_err(UserDomainError::Repo)?;
        match res {
            Some(_) => Ok(()),
            None => Err(UserDomainError::UserNotFound),
        }
    }
}
