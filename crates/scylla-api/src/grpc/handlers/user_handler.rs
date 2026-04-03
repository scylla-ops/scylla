use crate::extract_auth_context;
use crate::grpc::mappers::{
    domain_error_to_status, domain_to_proto_metadata, proto_to_domain_pagination, user_to_proto,
};
use derive_more::Constructor;
use protocol::services::user::{
    ChangeUserGlobalRoleRequest, ChangeUserGlobalRoleResponse, CreateUserRequest,
    DeleteUserRequest, DeleteUserResponse, GetUserRequest, ListUsersRequest, ListUsersResponse,
    UpdateUserRequest, UserResponse, user_service_server::UserService,
};
use scylla_core::application::UserUseCases;
use scylla_core::application::ports::services::permission_service::PermissionService;
use scylla_core::application::ports::{HashService, UserRepository};
use scylla_core::domain::entities::UserId;
use scylla_core::domain::value_objects::permission::policy;
use scylla_core::domain::value_objects::user::{Password, Username};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Constructor)]
pub struct UserHandler<U: UserRepository, H: HashService, PS: PermissionService> {
    use_cases: Arc<UserUseCases<U, H>>,
    permission_checker: Arc<PS>,
}
#[async_trait::async_trait]
impl<
    U: UserRepository + Send + Sync + 'static,
    H: HashService + Send + Sync + 'static,
    P: PermissionService + Send + Sync + 'static,
> UserService for UserHandler<U, H, P>
{
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        require_permission!(self, request, policy::user::create());
        let req = request.into_inner();

        let username = Username::new(&req.username).map_err(domain_error_to_status)?;
        let password = Password::new(&req.password).map_err(domain_error_to_status)?;

        let user = self
            .use_cases
            .create(username, password)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(user_to_proto(&user)))
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let target_user_id = UserId::new(&request.get_ref().user_id);
        require_permission!(self, request, policy::user::get(target_user_id.clone()));
        let _ = request.into_inner();

        let user = self
            .use_cases
            .get(&target_user_id)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(user_to_proto(&user)))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let target_user_id = UserId::new(&request.get_ref().user_id);
        require_permission!(self, request, policy::user::update(target_user_id.clone()));
        let req = request.into_inner();

        let username = req
            .username
            .map(|u| Username::new(&u))
            .transpose()
            .map_err(domain_error_to_status)?;
        let user = self
            .use_cases
            .update(&target_user_id, username)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(user_to_proto(&user)))
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let target_user_id = UserId::new(&request.get_ref().user_id);
        require_permission!(self, request, policy::user::delete(target_user_id.clone()));

        let _ = request.into_inner();

        self.use_cases
            .delete(&target_user_id)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(DeleteUserResponse {}))
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        require_permission!(self, request, policy::user::get_all());
        let req = request.into_inner();

        let pagination = proto_to_domain_pagination(req.pagination);
        let result = self
            .use_cases
            .list(pagination.as_ref())
            .await
            .map_err(domain_error_to_status)?;
        let (users, metadata) = result.into_parts();

        Ok(Response::new(ListUsersResponse {
            users: users.iter().map(user_to_proto).collect(),
            pagination: Some(domain_to_proto_metadata(&metadata)),
        }))
    }

    async fn change_user_global_role(
        &self,
        _request: Request<ChangeUserGlobalRoleRequest>,
    ) -> Result<Response<ChangeUserGlobalRoleResponse>, Status> {
        Err(Status::unimplemented(
            "Global role management requires RBAC configuration",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_interceptor::AuthContext;
    use protocol::services::user::user_service_server::UserService;
    use scylla_core::application::UserUseCases;
    use scylla_core::application::ports::{HashService, UserRepository};
    use scylla_core::application::ports::services::permission_service::PermissionService;
    use scylla_core::domain::entities::{EntityId, User};
    use scylla_core::domain::errors::{DomainError, DomainResult};
    use scylla_core::domain::value_objects::user::{Password, PasswordHash, Username};
    use scylla_core::domain::value_objects::permission::policy::{GroupingPolicy, Policy};
    use scylla_core::domain::value_objects::{PaginatedResult, PaginationParams};
    use async_trait::async_trait;
    use std::sync::Arc;

    // ── Stub UserRepo ───────────────────────────────────────────

    #[derive(Default)]
    struct StubUserRepo {
        create_fn: Option<Box<dyn Fn(&User) -> DomainResult<User> + Send + Sync>>,
        find_by_id_fn: Option<Box<dyn Fn(&UserId) -> DomainResult<User> + Send + Sync>>,
        update_fn: Option<Box<dyn Fn(&User) -> DomainResult<User> + Send + Sync>>,
        delete_fn: Option<Box<dyn Fn(&UserId) -> DomainResult<()> + Send + Sync>>,
        username_exists_fn: Option<Box<dyn Fn(&Username) -> DomainResult<bool> + Send + Sync>>,
        list_all_fn: Option<Box<dyn Fn() -> DomainResult<PaginatedResult<User>> + Send + Sync>>,
    }

    #[async_trait]
    impl UserRepository for StubUserRepo {
        async fn create(&self, u: &User) -> DomainResult<User> { (self.create_fn.as_ref().unwrap())(u) }
        async fn find_by_id(&self, id: &UserId) -> DomainResult<User> { (self.find_by_id_fn.as_ref().unwrap())(id) }
        async fn find_by_username(&self, _u: &Username) -> DomainResult<User> { unimplemented!() }
        async fn update(&self, u: &User) -> DomainResult<User> { (self.update_fn.as_ref().unwrap())(u) }
        async fn delete(&self, id: &UserId) -> DomainResult<()> { (self.delete_fn.as_ref().unwrap())(id) }
        async fn list_all(&self, _p: Option<&PaginationParams>) -> DomainResult<PaginatedResult<User>> {
            (self.list_all_fn.as_ref().unwrap())()
        }
        async fn username_exists(&self, u: &Username) -> DomainResult<bool> {
            (self.username_exists_fn.as_ref().unwrap())(u)
        }
    }

    // ── Stub HashService ────────────────────────────────────────

    struct StubHash;

    #[async_trait]
    impl HashService for StubHash {
        async fn hash(&self, _p: &Password) -> DomainResult<PasswordHash> {
            Ok(PasswordHash::new("$argon2id$v=19$m=19456,t=2,p=1$abc$def").unwrap())
        }
        async fn verify(&self, _p: &Password, _h: &PasswordHash) -> DomainResult<bool> { Ok(true) }
    }

    // ── Stub PermissionService (always allows) ──────────────────

    struct AllowAll;

    #[async_trait]
    impl PermissionService for AllowAll {
        async fn check(&self, _s: impl EntityId, _p: Policy) -> DomainResult<bool> { Ok(true) }
        async fn add_policy(&self, _s: impl EntityId, _p: Policy) -> DomainResult<bool> { Ok(true) }
        async fn remove_policy(&self, _s: impl EntityId, _p: Policy) -> DomainResult<bool> { Ok(true) }
        async fn list_policies(&self, _s: Option<&str>) -> DomainResult<Vec<(String, Policy)>> { Ok(vec![]) }
        async fn add_grouping_policy(&self, _s: impl EntityId, _p: GroupingPolicy) -> DomainResult<bool> { Ok(true) }
        async fn remove_grouping_policy(&self, _s: impl EntityId, _p: GroupingPolicy) -> DomainResult<bool> { Ok(true) }
        async fn list_grouping_policies(&self, _s: Option<&str>) -> DomainResult<Vec<(String, GroupingPolicy)>> { Ok(vec![]) }
    }

    // ── Helpers ──────────────────────────────────────────────────

    fn test_user() -> User {
        User::create(
            Username::new("testuser").unwrap(),
            PasswordHash::new("$argon2id$v=19$m=19456,t=2,p=1$abc$def").unwrap(),
        )
    }

    fn authed_request<T>(body: T) -> Request<T> {
        let mut req = Request::new(body);
        req.extensions_mut().insert(AuthContext::new(UserId::generate()));
        req
    }

    fn make_handler(repo: StubUserRepo) -> UserHandler<StubUserRepo, StubHash, AllowAll> {
        let uc = Arc::new(UserUseCases::new(Arc::new(repo), Arc::new(StubHash)));
        UserHandler::new(uc, Arc::new(AllowAll))
    }

    // ─��� Tests ────────────────────────────────────────────────────

    #[tokio::test]
    async fn create_user_returns_proto() {
        let mut repo = StubUserRepo::default();
        repo.username_exists_fn = Some(Box::new(|_| Ok(false)));
        repo.create_fn = Some(Box::new(|u| Ok(u.clone())));

        let handler = make_handler(repo);
        let req = authed_request(CreateUserRequest {
            username: "newuser".into(),
            password: "ValidPass123".into(),
        });

        let resp = handler.create_user(req).await.unwrap();
        let inner = resp.into_inner();
        assert_eq!(inner.username, "newuser");
        assert!(inner.is_active);
    }

    #[tokio::test]
    async fn get_user_returns_proto() {
        let user = test_user();
        let user_id_str = user.id().to_string();
        let u = user.clone();

        let mut repo = StubUserRepo::default();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(u.clone())));

        let handler = make_handler(repo);
        let req = authed_request(GetUserRequest { user_id: user_id_str });

        let resp = handler.get_user(req).await.unwrap();
        assert_eq!(resp.into_inner().username, "testuser");
    }

    #[tokio::test]
    async fn get_user_not_found() {
        let mut repo = StubUserRepo::default();
        repo.find_by_id_fn = Some(Box::new(|id| {
            Err(DomainError::not_found("User", id.to_string()))
        }));

        let handler = make_handler(repo);
        let req = authed_request(GetUserRequest { user_id: "nonexistent".into() });

        let err = handler.get_user(req).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn delete_user_success() {
        let user = test_user();
        let user_id_str = user.id().to_string();
        let u = user.clone();

        let mut repo = StubUserRepo::default();
        repo.find_by_id_fn = Some(Box::new(move |_| Ok(u.clone())));
        repo.delete_fn = Some(Box::new(|_| Ok(())));

        let handler = make_handler(repo);
        let req = authed_request(DeleteUserRequest { user_id: user_id_str });
        assert!(handler.delete_user(req).await.is_ok());
    }

    #[tokio::test]
    async fn list_users_returns_empty() {
        let mut repo = StubUserRepo::default();
        repo.list_all_fn = Some(Box::new(|| {
            let params = PaginationParams::default();
            Ok(PaginatedResult::new(vec![], &params, 0))
        }));

        let handler = make_handler(repo);
        let req = authed_request(ListUsersRequest { pagination: None });

        let resp = handler.list_users(req).await.unwrap();
        let inner = resp.into_inner();
        assert!(inner.users.is_empty());
        assert!(inner.pagination.is_some());
    }

    #[tokio::test]
    async fn create_user_without_auth_fails() {
        let repo = StubUserRepo::default();
        let handler = make_handler(repo);

        // No AuthContext inserted
        let req = Request::new(CreateUserRequest {
            username: "newuser".into(),
            password: "ValidPass123".into(),
        });

        let err = handler.create_user(req).await.unwrap_err();
        assert_eq!(err.code(), tonic::Code::Internal);
    }
}
