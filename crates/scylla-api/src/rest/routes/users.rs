use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use scylla_core::domain::entities::UserId;
use scylla_core::domain::value_objects::PaginationParams;
use scylla_core::domain::value_objects::permission::policy;
use scylla_core::domain::value_objects::user::{Password, Username};

use crate::rest::AppState;
use crate::rest::error::AppError;
use crate::rest::extract::Auth;
use crate::rest::response::*;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create).get(list))
        .route("/{id}", get(get_one).put(update).delete(delete_one))
        .route("/{id}/organizations", get(list_user_organizations))
        .route("/{id}/projects", get(list_user_projects))
}

#[utoipa::path(
    post, path = "/api/v1/users",
    tag = "Users",
    request_body = CreateUserRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "User created", body = UserResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 409, description = "User already exists", body = ErrorBody),
    )
)]
pub(crate) async fn create(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Json(body): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    require_permission(&state, &user_id, policy::user::create()).await?;

    let username = Username::new(&body.username).map_err(AppError::from)?;
    let password = Password::new(&body.password).map_err(AppError::from)?;

    let user = state.user_uc.create(username, password).await?;
    Ok(Json(UserResponse::from(&user)))
}

#[utoipa::path(
    get, path = "/api/v1/users/{id}",
    tag = "Users",
    params(("id" = String, Path, description = "User ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 404, description = "User not found", body = ErrorBody),
    )
)]
pub(crate) async fn get_one(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
) -> Result<Json<UserResponse>, AppError> {
    let target = UserId::new(&id);
    require_permission(&state, &user_id, policy::user::get(target.clone())).await?;

    let user = state.user_uc.get(&target).await?;
    Ok(Json(UserResponse::from(&user)))
}

#[utoipa::path(
    put, path = "/api/v1/users/{id}",
    tag = "Users",
    params(("id" = String, Path, description = "User ID")),
    request_body = UpdateUserRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "User updated", body = UserResponse),
        (status = 404, description = "User not found", body = ErrorBody),
    )
)]
pub(crate) async fn update(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
    Json(body): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let target = UserId::new(&id);
    require_permission(&state, &user_id, policy::user::update(target.clone())).await?;

    let username = body
        .username
        .map(|u| Username::new(&u))
        .transpose()
        .map_err(AppError::from)?;

    let user = state.user_uc.update(&target, username).await?;
    Ok(Json(UserResponse::from(&user)))
}

#[utoipa::path(
    delete, path = "/api/v1/users/{id}",
    tag = "Users",
    params(("id" = String, Path, description = "User ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "User deleted"),
        (status = 404, description = "User not found", body = ErrorBody),
    )
)]
pub(crate) async fn delete_one(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
) -> Result<Json<()>, AppError> {
    let target = UserId::new(&id);
    require_permission(&state, &user_id, policy::user::delete(target.clone())).await?;

    state.user_uc.delete(&target).await?;
    Ok(Json(()))
}

#[utoipa::path(
    get, path = "/api/v1/users",
    tag = "Users",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Items per page"),
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of users", body = ListUsersResponse),
    )
)]
pub(crate) async fn list(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ListUsersResponse>, AppError> {
    require_permission(&state, &user_id, policy::user::get_all()).await?;

    let params = to_pagination_params(&pagination);
    let result = state.user_uc.list(params.as_ref()).await?;
    let (users, metadata) = result.into_parts();

    Ok(Json(ListUsersResponse {
        users: users.iter().map(UserResponse::from).collect(),
        pagination: PaginationMeta::from(&metadata),
    }))
}

use scylla_core::application::ports::services::permission_service::PermissionService;
use scylla_core::domain::value_objects::permission::policy::Policy;

pub(crate) async fn require_permission(
    state: &AppState,
    user_id: &UserId,
    policy: Policy,
) -> Result<(), AppError> {
    state
        .permission_checker
        .check(user_id, policy)
        .await
        .map_err(AppError::from)?;
    Ok(())
}

pub(crate) fn to_pagination_params(q: &PaginationQuery) -> Option<PaginationParams> {
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(PaginationParams::DEFAULT_PAGE_SIZE);
    PaginationParams::new(page, page_size).ok()
}

#[utoipa::path(
    get, path = "/api/v1/users/{id}/organizations",
    tag = "Users",
    params(
        ("id" = String, Path, description = "User ID"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Items per page"),
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "User's organizations", body = ListUserOrganizationsResponse),
    )
)]
pub(crate) async fn list_user_organizations(
    State(state): State<AppState>,
    Auth(caller_id): Auth,
    Path(id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ListUserOrganizationsResponse>, AppError> {
    let target = UserId::new(&id);
    require_permission(
        &state,
        &caller_id,
        policy::organization::list_user_orgs(target.clone()),
    )
    .await?;

    let params = to_pagination_params(&pagination);
    let (orgs, metadata) = state
        .org_uc
        .list_user_orgs(&target, params.as_ref())
        .await?;

    Ok(Json(ListUserOrganizationsResponse {
        organizations: orgs.iter().map(OrganizationResponse::from).collect(),
        pagination: PaginationMeta::from(&metadata),
    }))
}

#[utoipa::path(
    get, path = "/api/v1/users/{id}/projects",
    tag = "Users",
    params(
        ("id" = String, Path, description = "User ID"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Items per page"),
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "User's projects", body = ListProjectsResponse),
    )
)]
pub(crate) async fn list_user_projects(
    State(state): State<AppState>,
    Auth(caller_id): Auth,
    Path(id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ListProjectsResponse>, AppError> {
    let target = UserId::new(&id);
    require_permission(
        &state,
        &caller_id,
        policy::project::list_user_projects(target.clone()),
    )
    .await?;

    let params = to_pagination_params(&pagination);
    let (projects, metadata) = state
        .project_uc
        .list_user_projects(&target, params.as_ref())
        .await?;

    Ok(Json(ListProjectsResponse {
        projects: projects.iter().map(ProjectResponse::from).collect(),
        pagination: PaginationMeta::from(&metadata),
    }))
}
