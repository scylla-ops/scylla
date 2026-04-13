//! E2E integration tests for auth and user flows.
//!
//! Starts a real gRPC server with in-memory SurrealDB and exercises the full
//! request path: client → gRPC transport → interceptor → handler → use case → repo.

use protocol::services::{
    auth::{
        LoginRequest, RevokeTokenRequest, ValidateTokenRequest,
        auth_service_client::AuthServiceClient,
    },
    user::{
        CreateUserRequest, DeleteUserRequest, GetUserRequest, ListUsersRequest, UpdateUserRequest,
        user_service_client::UserServiceClient,
    },
};
use std::net::SocketAddr;
use tokio::task::JoinHandle;
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;

/// Spin up an in-memory server and return the address + join handle.
async fn spawn_test_server() -> (SocketAddr, JoinHandle<()>) {
    use protocol::services::{
        auth::auth_service_server::AuthServiceServer, user::user_service_server::UserServiceServer,
    };
    use scylla_api::{AuthHandler, UserHandler, auth_interceptor::AuthInterceptor};
    use scylla_core::application::{AuthUseCases, UserUseCases};
    use scylla_core::infrastructure::{
        Argon2HashService, CasbinPermissionService, SurrealSessionRepository, SurrealUserRepository,
    };
    use surreal_casbin_adapter::SurrealAdapter;
    use tonic::transport::Server;
    use tonic_async_interceptor::async_interceptor;
    use tower::ServiceBuilder;

    let db_config = scylla_core::infrastructure::DatabaseConfig::default();
    let db = scylla_core::infrastructure::init_db(&db_config)
        .await
        .unwrap();

    let user_repo = std::sync::Arc::new(SurrealUserRepository::new(db.clone()));
    let session_repo = std::sync::Arc::new(SurrealSessionRepository::new(db.clone()));
    let hash_service = std::sync::Arc::new(Argon2HashService::new());

    let auth_uc = std::sync::Arc::new(AuthUseCases::new(
        user_repo.clone(),
        session_repo.clone(),
        hash_service.clone(),
    ));
    let user_uc = std::sync::Arc::new(UserUseCases::new(user_repo.clone(), hash_service.clone()));

    let surreal_casbin_adapter = SurrealAdapter::new(db.clone());
    let casbin_service = CasbinPermissionService::new(surreal_casbin_adapter)
        .await
        .unwrap();
    let permission_checker = std::sync::Arc::new(casbin_service);

    let auth_handler = AuthHandler::new(auth_uc);
    let user_handler = UserHandler::new(user_uc, permission_checker);

    let auth_interceptor = async_interceptor(AuthInterceptor::new(session_repo));

    // Bind to port 0 for auto-assignment
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let handle = tokio::spawn(async move {
        let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);

        let auth_service = AuthServiceServer::new(auth_handler);
        let user_service = ServiceBuilder::new()
            .layer(auth_interceptor)
            .service(UserServiceServer::new(user_handler));

        Server::builder()
            .add_service(auth_service)
            .add_service(user_service)
            .serve_with_incoming(incoming)
            .await
            .unwrap();
    });

    // Give server a moment to start
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    (addr, handle)
}

async fn connect(addr: SocketAddr) -> Channel {
    Channel::from_shared(format!("http://{addr}"))
        .unwrap()
        .connect()
        .await
        .unwrap()
}

// ── Auth flow tests ──────────────────────────────────────────────

#[tokio::test]
async fn e2e_auth_login_validate_revoke() {
    let (addr, _handle) = spawn_test_server().await;
    let channel = connect(addr).await;

    let mut auth = AuthServiceClient::new(channel.clone());
    // Login with nonexistent user should fail
    let err = auth
        .login(LoginRequest {
            username: "nonexistent".into(),
            password: "ValidPass123".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::Unauthenticated);

    // Validate empty token returns false
    let resp = auth
        .validate_token(ValidateTokenRequest {
            token: String::new(),
        })
        .await
        .unwrap();
    assert_eq!(resp.into_inner().is_valid, Some(false));

    // Revoke empty token returns error
    let err = auth
        .revoke_token(RevokeTokenRequest {
            token: String::new(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn e2e_auth_full_flow_with_bootstrap() {
    use protocol::services::{
        auth::auth_service_server::AuthServiceServer, user::user_service_server::UserServiceServer,
    };
    use scylla_api::{AuthHandler, UserHandler, auth_interceptor::AuthInterceptor};
    use scylla_core::application::{AuthUseCases, UserUseCases};
    use scylla_core::domain::value_objects::user::{Password, Username};
    use scylla_core::infrastructure::{
        Argon2HashService, CasbinPermissionService, SurrealSessionRepository, SurrealUserRepository,
    };
    use surreal_casbin_adapter::SurrealAdapter;
    use tonic::transport::Server;
    use tonic_async_interceptor::async_interceptor;
    use tower::ServiceBuilder;

    // Set up DB and repos
    let db_config = scylla_core::infrastructure::DatabaseConfig::default();
    let db = scylla_core::infrastructure::init_db(&db_config)
        .await
        .unwrap();

    let user_repo = std::sync::Arc::new(SurrealUserRepository::new(db.clone()));
    let session_repo = std::sync::Arc::new(SurrealSessionRepository::new(db.clone()));
    let hash_service = std::sync::Arc::new(Argon2HashService::new());

    let auth_uc = std::sync::Arc::new(AuthUseCases::new(
        user_repo.clone(),
        session_repo.clone(),
        hash_service.clone(),
    ));
    let user_uc = std::sync::Arc::new(UserUseCases::new(user_repo.clone(), hash_service.clone()));

    let surreal_casbin_adapter = SurrealAdapter::new(db.clone());
    let casbin_service = CasbinPermissionService::new(surreal_casbin_adapter)
        .await
        .unwrap();
    let permission_checker = std::sync::Arc::new(casbin_service);

    // Bootstrap: create admin user directly via use case
    let admin_username = Username::new("admin").unwrap();
    let admin_password = Password::new("AdminPass123").unwrap();
    user_uc
        .create(admin_username.clone(), admin_password.clone())
        .await
        .unwrap();

    let auth_handler = AuthHandler::new(auth_uc);
    let user_handler = UserHandler::new(user_uc, permission_checker);
    let auth_interceptor = async_interceptor(AuthInterceptor::new(session_repo));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let _handle = tokio::spawn(async move {
        let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);
        Server::builder()
            .add_service(AuthServiceServer::new(auth_handler))
            .add_service(
                ServiceBuilder::new()
                    .layer(auth_interceptor)
                    .service(UserServiceServer::new(user_handler)),
            )
            .serve_with_incoming(incoming)
            .await
            .unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let channel = connect(addr).await;
    let mut auth = AuthServiceClient::new(channel.clone());

    // 1. Login
    let login_resp = auth
        .login(LoginRequest {
            username: "admin".into(),
            password: "AdminPass123".into(),
        })
        .await
        .unwrap()
        .into_inner();
    assert!(!login_resp.token.is_empty());
    assert!(!login_resp.user_id.is_empty());

    let token = login_resp.token.clone();

    // 2. Validate token
    let valid_resp = auth
        .validate_token(ValidateTokenRequest {
            token: token.clone(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(valid_resp.is_valid, Some(true));

    // 3. Revoke token
    auth.revoke_token(RevokeTokenRequest {
        token: token.clone(),
    })
    .await
    .unwrap();

    // 4. Token should no longer be valid
    let invalid_resp = auth
        .validate_token(ValidateTokenRequest { token })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(invalid_resp.is_valid, Some(false));
}

#[tokio::test]
async fn e2e_user_crud_with_auth() {
    use protocol::services::{
        auth::auth_service_server::AuthServiceServer, user::user_service_server::UserServiceServer,
    };
    use scylla_api::{AuthHandler, UserHandler, auth_interceptor::AuthInterceptor};
    use scylla_core::application::ports::services::permission_service::PermissionService;
    use scylla_core::application::{AuthUseCases, UserUseCases};
    use scylla_core::domain::value_objects::permission::policy::Policy;
    use scylla_core::domain::value_objects::permission::{Act, Resource, Scope};
    use scylla_core::domain::value_objects::user::{Password, Username};
    use scylla_core::infrastructure::{
        Argon2HashService, CasbinPermissionService, SurrealSessionRepository, SurrealUserRepository,
    };
    use surreal_casbin_adapter::SurrealAdapter;
    use tonic::transport::Server;
    use tonic_async_interceptor::async_interceptor;
    use tower::ServiceBuilder;

    // Set up DB and repos
    let db_config = scylla_core::infrastructure::DatabaseConfig::default();
    let db = scylla_core::infrastructure::init_db(&db_config)
        .await
        .unwrap();

    let user_repo = std::sync::Arc::new(SurrealUserRepository::new(db.clone()));
    let session_repo = std::sync::Arc::new(SurrealSessionRepository::new(db.clone()));
    let hash_service = std::sync::Arc::new(Argon2HashService::new());

    let auth_uc = std::sync::Arc::new(AuthUseCases::new(
        user_repo.clone(),
        session_repo.clone(),
        hash_service.clone(),
    ));
    let user_uc = std::sync::Arc::new(UserUseCases::new(user_repo.clone(), hash_service.clone()));

    let surreal_casbin_adapter = SurrealAdapter::new(db.clone());
    let casbin_service = CasbinPermissionService::new(surreal_casbin_adapter)
        .await
        .unwrap();

    // Bootstrap admin user
    let admin = user_uc
        .create(
            Username::new("admin").unwrap(),
            Password::new("AdminPass123").unwrap(),
        )
        .await
        .unwrap();

    // Grant admin all permissions on all resources
    let admin_id = admin.id().clone();
    casbin_service
        .add_policy(
            admin_id.clone(),
            Policy::new(Scope::All, Resource::All, Act::All),
        )
        .await
        .unwrap();

    let permission_checker = std::sync::Arc::new(casbin_service);

    let auth_handler = AuthHandler::new(auth_uc);
    let user_handler = UserHandler::new(user_uc, permission_checker);
    let auth_interceptor = async_interceptor(AuthInterceptor::new(session_repo));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let _handle = tokio::spawn(async move {
        let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener);
        Server::builder()
            .add_service(AuthServiceServer::new(auth_handler))
            .add_service(
                ServiceBuilder::new()
                    .layer(auth_interceptor)
                    .service(UserServiceServer::new(user_handler)),
            )
            .serve_with_incoming(incoming)
            .await
            .unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let channel = connect(addr).await;
    let mut auth = AuthServiceClient::new(channel.clone());

    // Login as admin
    let login_resp = auth
        .login(LoginRequest {
            username: "admin".into(),
            password: "AdminPass123".into(),
        })
        .await
        .unwrap()
        .into_inner();

    let token = login_resp.token;
    let bearer: MetadataValue<_> = format!("Bearer {token}").parse().unwrap();

    // Create an intercepted user client
    let mut users =
        UserServiceClient::with_interceptor(channel.clone(), move |mut req: tonic::Request<()>| {
            req.metadata_mut().insert("authorization", bearer.clone());
            Ok(req)
        });

    // 1. Create user
    let create_resp = users
        .create_user(CreateUserRequest {
            username: "newuser".into(),
            password: "NewPass123".into(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(create_resp.username, "newuser");
    assert!(create_resp.is_active);
    let user_id = create_resp.user_id.clone();

    // 2. Get user
    let get_resp = users
        .get_user(GetUserRequest {
            user_id: user_id.clone(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(get_resp.username, "newuser");

    // 3. Update user
    let update_resp = users
        .update_user(UpdateUserRequest {
            user_id: user_id.clone(),
            username: Some("updateduser".into()),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(update_resp.username, "updateduser");

    // 4. List users (should have admin + newuser)
    let list_resp = users
        .list_users(ListUsersRequest { pagination: None })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(list_resp.users.len(), 2);

    // 5. Delete user
    users
        .delete_user(DeleteUserRequest {
            user_id: user_id.clone(),
        })
        .await
        .unwrap();

    // 6. Get deleted user should fail
    let err = users
        .get_user(GetUserRequest { user_id })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::NotFound);
}
