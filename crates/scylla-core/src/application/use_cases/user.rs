use crate::application::ports::{HashService, UserRepository};
use crate::domain::entities::{User, UserId};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::user::{Password, Username};
use crate::domain::value_objects::{PaginatedResult, PaginationParams};
use derive_more::Constructor;
use std::sync::Arc;
use tracing::instrument;

#[derive(Constructor)]
pub struct UserUseCases<U: UserRepository, H: HashService> {
    user_repo: Arc<U>,
    hash_service: Arc<H>,
}

impl<U: UserRepository, H: HashService> UserUseCases<U, H> {
    #[instrument(skip(self, password), fields(username = %username))]
    pub async fn create(&self, username: Username, password: Password) -> DomainResult<User> {
        if self.user_repo.username_exists(&username).await? {
            return Err(DomainError::conflict("Username already exists"));
        }

        let password_hash = self.hash_service.hash(&password).await?;
        let user = User::create(username, password_hash);
        self.user_repo.create(&user).await
    }

    #[instrument(skip(self), fields(user_id = %id))]
    pub async fn get(&self, id: &UserId) -> DomainResult<User> {
        self.user_repo.find_by_id(id).await
    }

    #[instrument(skip(self), fields(user_id = %id))]
    pub async fn update(&self, id: &UserId, username: Option<Username>) -> DomainResult<User> {
        let mut user = self.user_repo.find_by_id(id).await?;

        if let Some(new_username) = username {
            if self.user_repo.username_exists(&new_username).await?
                && user.username() != &new_username
            {
                return Err(DomainError::conflict("Username already exists"));
            }
            user.update_username(new_username)?;
        }

        self.user_repo.update(&user).await
    }

    #[instrument(skip(self), fields(user_id = %id))]
    pub async fn delete(&self, id: &UserId) -> DomainResult<()> {
        self.user_repo.find_by_id(id).await?;
        self.user_repo.delete(id).await
    }

    #[instrument(skip(self))]
    pub async fn list(
        &self,
        pagination: Option<&PaginationParams>,
    ) -> DomainResult<PaginatedResult<User>> {
        self.user_repo.list_all(pagination).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{HashService, UserRepository};
    use crate::domain::value_objects::user::PasswordHash;
    use async_trait::async_trait;
    use std::sync::Arc;

    // ── Stub UserRepository ──────────────────────────────────────

    #[derive(Default)]
    struct StubUserRepo {
        create_fn: Option<Box<dyn Fn(&User) -> DomainResult<User> + Send + Sync>>,
        find_by_id_fn: Option<Box<dyn Fn(&UserId) -> DomainResult<User> + Send + Sync>>,
        find_by_username_fn: Option<Box<dyn Fn(&Username) -> DomainResult<User> + Send + Sync>>,
        update_fn: Option<Box<dyn Fn(&User) -> DomainResult<User> + Send + Sync>>,
        delete_fn: Option<Box<dyn Fn(&UserId) -> DomainResult<()> + Send + Sync>>,
        list_all_fn: Option<Box<dyn Fn() -> DomainResult<PaginatedResult<User>> + Send + Sync>>,
        username_exists_fn: Option<Box<dyn Fn(&Username) -> DomainResult<bool> + Send + Sync>>,
    }

    #[async_trait]
    impl UserRepository for StubUserRepo {
        async fn create(&self, user: &User) -> DomainResult<User> {
            (self.create_fn.as_ref().unwrap())(user)
        }
        async fn find_by_id(&self, id: &UserId) -> DomainResult<User> {
            (self.find_by_id_fn.as_ref().unwrap())(id)
        }
        async fn find_by_username(&self, username: &Username) -> DomainResult<User> {
            (self.find_by_username_fn.as_ref().unwrap())(username)
        }
        async fn update(&self, user: &User) -> DomainResult<User> {
            (self.update_fn.as_ref().unwrap())(user)
        }
        async fn delete(&self, id: &UserId) -> DomainResult<()> {
            (self.delete_fn.as_ref().unwrap())(id)
        }
        async fn list_all(
            &self,
            _pagination: Option<&PaginationParams>,
        ) -> DomainResult<PaginatedResult<User>> {
            (self.list_all_fn.as_ref().unwrap())()
        }
        async fn username_exists(&self, username: &Username) -> DomainResult<bool> {
            (self.username_exists_fn.as_ref().unwrap())(username)
        }
    }

    // ── Stub HashService ─────────────────────────────────────────

    #[derive(Default)]
    struct StubHash {
        hash_fn: Option<Box<dyn Fn(&Password) -> DomainResult<PasswordHash> + Send + Sync>>,
    }

    #[async_trait]
    impl HashService for StubHash {
        async fn hash(&self, password: &Password) -> DomainResult<PasswordHash> {
            (self.hash_fn.as_ref().unwrap())(password)
        }
        async fn verify(&self, _password: &Password, _hash: &PasswordHash) -> DomainResult<bool> {
            unimplemented!()
        }
    }

    // ── Helpers ──────────────────────────────────────────────────

    fn test_user() -> User {
        let username = Username::new("testuser").unwrap();
        let hash = PasswordHash::new("$argon2id$v=19$m=19456,t=2,p=1$abc$def").unwrap();
        User::create(username, hash)
    }

    fn fake_hash() -> PasswordHash {
        PasswordHash::new("$argon2id$v=19$m=19456,t=2,p=1$abc$def").unwrap()
    }

    fn make_uc(
        user_repo: StubUserRepo,
        hash_service: StubHash,
    ) -> UserUseCases<StubUserRepo, StubHash> {
        UserUseCases::new(Arc::new(user_repo), Arc::new(hash_service))
    }

    // ── Tests ────────────────────────────────────────────────────

    #[tokio::test]
    async fn create_success() {
        let mut repo = StubUserRepo::default();
        repo.username_exists_fn = Some(Box::new(|_| Ok(false)));
        repo.create_fn = Some(Box::new(|u| Ok(u.clone())));

        let mut hash = StubHash::default();
        hash.hash_fn = Some(Box::new(|_| Ok(fake_hash())));

        let uc = make_uc(repo, hash);
        let username = Username::new("newuser").unwrap();
        let password = Password::new("ValidPass123").unwrap();

        let user = uc.create(username, password).await.unwrap();
        assert_eq!(user.username().as_str(), "newuser");
        assert!(user.is_active());
    }

    #[tokio::test]
    async fn create_duplicate_username() {
        let mut repo = StubUserRepo::default();
        repo.username_exists_fn = Some(Box::new(|_| Ok(true)));

        let uc = make_uc(repo, StubHash::default());
        let username = Username::new("existing").unwrap();
        let password = Password::new("ValidPass123").unwrap();

        let result = uc.create(username, password).await;
        assert!(matches!(result.unwrap_err(), DomainError::Conflict(_)));
    }

    #[tokio::test]
    async fn get_existing_user() {
        let user = test_user();
        let mut repo = StubUserRepo::default();
        let u = user.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(u.clone())));

        let uc = make_uc(repo, StubHash::default());
        let result = uc.get(user.id()).await.unwrap();
        assert_eq!(result.username().as_str(), "testuser");
    }

    #[tokio::test]
    async fn get_nonexistent_user() {
        let mut repo = StubUserRepo::default();
        repo.find_by_id_fn = Some(Box::new(|id| {
            Err(DomainError::not_found("User", id.to_string()))
        }));

        let uc = make_uc(repo, StubHash::default());
        let id = UserId::generate();
        let result = uc.get(&id).await;
        assert!(matches!(result.unwrap_err(), DomainError::NotFound { .. }));
    }

    #[tokio::test]
    async fn update_username() {
        let user = test_user();
        let mut repo = StubUserRepo::default();
        let u = user.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(u.clone())));
        repo.username_exists_fn = Some(Box::new(|_| Ok(false)));
        repo.update_fn = Some(Box::new(|u| Ok(u.clone())));

        let uc = make_uc(repo, StubHash::default());
        let new_name = Username::new("newname").unwrap();
        let result = uc.update(user.id(), Some(new_name)).await.unwrap();
        assert_eq!(result.username().as_str(), "newname");
    }

    #[tokio::test]
    async fn update_username_conflict() {
        let user = test_user();
        let mut repo = StubUserRepo::default();
        let u = user.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(u.clone())));
        repo.username_exists_fn = Some(Box::new(|_| Ok(true)));

        let uc = make_uc(repo, StubHash::default());
        let new_name = Username::new("taken").unwrap();
        let result = uc.update(user.id(), Some(new_name)).await;
        assert!(matches!(result.unwrap_err(), DomainError::Conflict(_)));
    }

    #[tokio::test]
    async fn update_same_username_no_conflict() {
        let user = test_user();
        let mut repo = StubUserRepo::default();
        let u = user.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(u.clone())));
        repo.username_exists_fn = Some(Box::new(|_| Ok(true)));
        repo.update_fn = Some(Box::new(|u| Ok(u.clone())));

        let uc = make_uc(repo, StubHash::default());
        let same_name = Username::new("testuser").unwrap();
        let result = uc.update(user.id(), Some(same_name)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn update_no_changes() {
        let user = test_user();
        let mut repo = StubUserRepo::default();
        let u = user.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(u.clone())));
        repo.update_fn = Some(Box::new(|u| Ok(u.clone())));

        let uc = make_uc(repo, StubHash::default());
        let result = uc.update(user.id(), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_success() {
        let user = test_user();
        let user_id = user.id().clone();

        let mut repo = StubUserRepo::default();
        let u = user.clone();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(u.clone())));
        repo.delete_fn = Some(Box::new(|_| Ok(())));

        let uc = make_uc(repo, StubHash::default());
        assert!(uc.delete(&user_id).await.is_ok());
    }

    #[tokio::test]
    async fn delete_nonexistent() {
        let mut repo = StubUserRepo::default();
        repo.find_by_id_fn = Some(Box::new(|id| {
            Err(DomainError::not_found("User", id.to_string()))
        }));

        let uc = make_uc(repo, StubHash::default());
        let id = UserId::generate();
        let result = uc.delete(&id).await;
        assert!(matches!(result.unwrap_err(), DomainError::NotFound { .. }));
    }

    #[tokio::test]
    async fn list_users() {
        let mut repo = StubUserRepo::default();
        repo.list_all_fn = Some(Box::new(|| {
            let params = PaginationParams::default();
            Ok(PaginatedResult::new(vec![], &params, 0))
        }));

        let uc = make_uc(repo, StubHash::default());
        let result = uc.list(None).await.unwrap();
        assert_eq!(result.metadata().total_count(), 0);
        assert!(result.items().is_empty());
    }
}
