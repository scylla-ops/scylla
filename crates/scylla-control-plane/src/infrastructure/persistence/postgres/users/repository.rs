use crate::application::UserRepository;
use crate::application::pagination::{PaginatedResult, PaginationParams};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ids::UserId;
use crate::domain::user::User;
use crate::domain::user::{Email, PasswordHash, Username};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use tracing::instrument;

use super::super::error::{DbFieldExt, SqlxResultExt};

#[derive(Clone)]
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    #[instrument(skip(self, user), fields(user_id = %user.id()))]
    async fn create(&self, user: &User) -> DomainResult<User> {
        queries::create(&self.pool, user).await
    }

    #[instrument(skip(self), fields(user_id = %id))]
    async fn find_by_id(&self, id: &UserId) -> DomainResult<User> {
        queries::find_by_id(&self.pool, id).await
    }

    #[instrument(skip(self, ids), fields(n = ids.len()))]
    async fn find_by_ids(&self, ids: &[UserId]) -> DomainResult<Vec<User>> {
        queries::find_by_ids(&self.pool, ids).await
    }

    #[instrument(skip(self), fields(username = %username))]
    async fn find_by_username(&self, username: &Username) -> DomainResult<User> {
        queries::find_by_username(&self.pool, username).await
    }

    #[instrument(skip(self), fields(email = %email))]
    async fn find_by_email(&self, email: &Email) -> DomainResult<User> {
        queries::find_by_email(&self.pool, email).await
    }

    #[instrument(skip(self, user), fields(user_id = %user.id()))]
    async fn update(&self, user: &User) -> DomainResult<User> {
        queries::update(&self.pool, user).await
    }

    #[instrument(skip(self), fields(user_id = %id))]
    async fn delete(&self, id: &UserId) -> DomainResult<()> {
        queries::delete(&self.pool, id).await
    }

    #[instrument(skip(self))]
    async fn list_all(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<User>> {
        let params = pagination.copied().unwrap_or_default();
        let total = queries::count_all(&self.pool).await?;
        let items = queries::list_page(&self.pool, &params).await?;
        Ok(PaginatedResult::new(items, &params, total))
    }

    #[instrument(skip(self), fields(username = %username))]
    async fn username_exists(&self, username: &Username) -> DomainResult<bool> {
        queries::username_exists(&self.pool, username).await
    }
}

#[allow(clippy::wildcard_imports)]
pub mod queries {
    use super::*;

    #[allow(clippy::too_many_arguments)]
    fn row_into_user(
        id: String,
        username: String,
        email: Option<String>,
        password_hash: String,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> DomainResult<User> {
        let username = Username::new(username).db_field("username")?;
        let email = email.map(Email::new).transpose().db_field("email")?;
        let password_hash = PasswordHash::new(password_hash).db_field("password hash")?;
        Ok(User::from_persistence(
            UserId::new(id),
            username,
            email,
            password_hash,
            is_active,
            created_at,
            updated_at,
        ))
    }

    pub async fn create<'e, E>(executor: E, user: &User) -> DomainResult<User>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!(
            r#"
            INSERT INTO users (id, username, email, password_hash, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            user.id().as_str(),
            user.username().as_str(),
            user.email().map(Email::as_str),
            user.password_hash().as_str(),
            user.is_active(),
            user.created_at(),
            user.updated_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        Ok(user.clone())
    }

    pub async fn find_by_id<'e, E>(executor: E, id: &UserId) -> DomainResult<User>
    where
        E: PgExecutor<'e>,
    {
        let rec = sqlx::query!(
            r#"
            SELECT id, username, email, password_hash, is_active, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id.as_str(),
        )
        .fetch_one(executor)
        .await
        .not_found_as("User", id.to_string())?;
        row_into_user(
            rec.id,
            rec.username,
            rec.email,
            rec.password_hash,
            rec.is_active,
            rec.created_at,
            rec.updated_at,
        )
    }

    pub async fn find_by_ids<'e, E>(executor: E, ids: &[UserId]) -> DomainResult<Vec<User>>
    where
        E: PgExecutor<'e>,
    {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let id_strs: Vec<String> = ids.iter().map(|i| i.as_str().to_owned()).collect();
        let rows = sqlx::query!(
            r#"
            SELECT id, username, email, password_hash, is_active, created_at, updated_at
            FROM users
            WHERE id = ANY($1::text[])
            "#,
            &id_strs,
        )
        .fetch_all(executor)
        .await
        .to_domain()?;
        rows.into_iter()
            .map(|r| {
                row_into_user(
                    r.id,
                    r.username,
                    r.email,
                    r.password_hash,
                    r.is_active,
                    r.created_at,
                    r.updated_at,
                )
            })
            .collect()
    }

    pub async fn find_by_username<'e, E>(executor: E, username: &Username) -> DomainResult<User>
    where
        E: PgExecutor<'e>,
    {
        let rec = sqlx::query!(
            r#"
            SELECT id, username, email, password_hash, is_active, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
            username.as_str(),
        )
        .fetch_one(executor)
        .await
        // Don't echo the looked-up username (PII / account-existence oracle on
        // login paths) into the error id.
        .not_found_as("User", "<username>")?;
        row_into_user(
            rec.id,
            rec.username,
            rec.email,
            rec.password_hash,
            rec.is_active,
            rec.created_at,
            rec.updated_at,
        )
    }

    pub async fn find_by_email<'e, E>(executor: E, email: &Email) -> DomainResult<User>
    where
        E: PgExecutor<'e>,
    {
        let rec = sqlx::query!(
            r#"
            SELECT id, username, email, password_hash, is_active, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email.as_str(),
        )
        .fetch_one(executor)
        .await
        // Don't echo the looked-up email (PII / account-existence oracle) into
        // the error id.
        .not_found_as("User", "<email>")?;
        row_into_user(
            rec.id,
            rec.username,
            rec.email,
            rec.password_hash,
            rec.is_active,
            rec.created_at,
            rec.updated_at,
        )
    }

    pub async fn update<'e, E>(executor: E, user: &User) -> DomainResult<User>
    where
        E: PgExecutor<'e>,
    {
        let res = sqlx::query!(
            r#"
            UPDATE users
            SET username = $2,
                email = $3,
                password_hash = $4,
                is_active = $5,
                updated_at = $6
            WHERE id = $1
            "#,
            user.id().as_str(),
            user.username().as_str(),
            user.email().map(Email::as_str),
            user.password_hash().as_str(),
            user.is_active(),
            user.updated_at(),
        )
        .execute(executor)
        .await
        .to_domain()?;
        if res.rows_affected() == 0 {
            return Err(DomainError::not_found("User", user.id().to_string()));
        }
        Ok(user.clone())
    }

    pub async fn delete<'e, E>(executor: E, id: &UserId) -> DomainResult<()>
    where
        E: PgExecutor<'e>,
    {
        sqlx::query!("DELETE FROM users WHERE id = $1", id.as_str())
            .execute(executor)
            .await
            .to_domain()?;
        Ok(())
    }

    pub async fn count_all<'e, E>(executor: E) -> DomainResult<u64>
    where
        E: PgExecutor<'e>,
    {
        let row = sqlx::query!(r#"SELECT COUNT(*) AS "count!" FROM users"#)
            .fetch_one(executor)
            .await
            .to_domain()?;
        Ok(u64::try_from(row.count).unwrap_or(0))
    }

    pub async fn list_page<'e, E>(executor: E, params: &PaginationParams) -> DomainResult<Vec<User>>
    where
        E: PgExecutor<'e>,
    {
        let limit = i64::try_from(params.limit()).unwrap_or(i64::MAX);
        let offset = i64::try_from(params.offset()).unwrap_or(i64::MAX);
        let rows = sqlx::query!(
            r#"
            SELECT id, username, email, password_hash, is_active, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
        )
        .fetch_all(executor)
        .await
        .to_domain()?;
        rows.into_iter()
            .map(|r| {
                row_into_user(
                    r.id,
                    r.username,
                    r.email,
                    r.password_hash,
                    r.is_active,
                    r.created_at,
                    r.updated_at,
                )
            })
            .collect()
    }

    pub async fn username_exists<'e, E>(executor: E, username: &Username) -> DomainResult<bool>
    where
        E: PgExecutor<'e>,
    {
        let row = sqlx::query!(
            r#"SELECT EXISTS(SELECT 1 FROM users WHERE username = $1) AS "exists!""#,
            username.as_str(),
        )
        .fetch_one(executor)
        .await
        .to_domain()?;
        Ok(row.exists)
    }
}
