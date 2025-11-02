use crate::application::dto::{CreateUserRequestDto, UserResponseDto};
use crate::application::ports::{PasswordHasher, RbacEnforcer};
use crate::domain::entities::User;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::repositories::UserRepository;
use crate::domain::value_objects::UserGlobalRole;
use crate::infrastructure::rbac::RoleMapper;
use std::sync::Arc;

pub struct CreateUserUseCase<R, H, E>
where
    R: UserRepository + ?Sized,
    H: PasswordHasher + ?Sized,
    E: RbacEnforcer + ?Sized,
{
    user_repo: Arc<R>,
    password_hasher: Arc<H>,
    rbac_enforcer: Arc<E>,
}

impl<R, H, E> CreateUserUseCase<R, H, E>
where
    R: UserRepository + ?Sized,
    H: PasswordHasher + ?Sized,
    E: RbacEnforcer + ?Sized,
{
    pub fn new(user_repo: Arc<R>, password_hasher: Arc<H>, rbac_enforcer: Arc<E>) -> Self {
        Self {
            user_repo,
            password_hasher,
            rbac_enforcer,
        }
    }

    pub async fn execute(&self, request: CreateUserRequestDto) -> DomainResult<UserResponseDto> {
        // check if username already exists
        if self.user_repo.username_exists(&request.username).await? {
            return Err(DomainError::conflict(format!(
                "Username '{}' already exists",
                request.username
            )));
        }

        let password_hash = self.password_hasher.hash(&request.password).await?;

        let user_draft = User::create(request.username, password_hash);
        let user_created = self.user_repo.create(&user_draft).await?;

        let global_role = UserGlobalRole::user();
        let casbin_role = RoleMapper::global_role_to_casbin(&global_role);

        self.rbac_enforcer
            .add_role_for_user(user_created.id(), casbin_role, "*")
            .await
            .map_err(|e| {
                DomainError::internal(format!("Failed to assign global role to user: {}", e))
            })?;

        Ok(UserResponseDto::from(user_created))
    }
}
