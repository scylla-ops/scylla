use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use scylla_core::domain::entities::{OrganizationId, UserId};
use scylla_core::domain::value_objects::organization::{OrganizationDescription, OrganizationName};
use scylla_core::domain::value_objects::permission::policy;

use crate::rest::AppState;
use crate::rest::error::AppError;
use crate::rest::extract::Auth;
use crate::rest::response::*;

use super::users::{require_permission, to_pagination_params};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create).get(list))
        .route("/{id}", get(get_one).put(update).delete(delete_one))
        .route("/{id}/toggle-active", post(toggle_active))
        .route("/{id}/users", get(list_users).post(add_user))
        .route("/{id}/users/{user_id}", axum::routing::delete(remove_user))
        .route("/{id}/pipelines", get(list_pipelines))
        .route("/{id}/jobs", get(list_jobs))
}

#[utoipa::path(
    post, path = "/api/v1/organizations",
    tag = "Organizations",
    request_body = CreateOrganizationRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Organization created", body = OrganizationResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
    )
)]
pub(crate) async fn create(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Json(body): Json<CreateOrganizationRequest>,
) -> Result<Json<OrganizationResponse>, AppError> {
    require_permission(&state, &user_id, policy::organization::create()).await?;

    let name = OrganizationName::new(&body.name).map_err(AppError::from)?;
    let description = body
        .description
        .as_deref()
        .map(OrganizationDescription::new)
        .transpose()
        .map_err(AppError::from)?;

    let org = state.org_uc.create(name, description).await?;
    Ok(Json(OrganizationResponse::from(&org)))
}

#[utoipa::path(
    get, path = "/api/v1/organizations/{id}",
    tag = "Organizations",
    params(("id" = String, Path, description = "Organization ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Organization found", body = OrganizationResponse),
        (status = 404, description = "Not found", body = ErrorBody),
    )
)]
pub(crate) async fn get_one(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
) -> Result<Json<OrganizationResponse>, AppError> {
    let target = OrganizationId::new(&id);
    require_permission(&state, &user_id, policy::organization::get(target.clone())).await?;

    let org = state.org_uc.get(&target).await?;
    Ok(Json(OrganizationResponse::from(&org)))
}

#[utoipa::path(
    put, path = "/api/v1/organizations/{id}",
    tag = "Organizations",
    params(("id" = String, Path, description = "Organization ID")),
    request_body = UpdateOrganizationRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Organization updated", body = OrganizationResponse),
        (status = 404, description = "Not found", body = ErrorBody),
    )
)]
pub(crate) async fn update(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
    Json(body): Json<UpdateOrganizationRequest>,
) -> Result<Json<OrganizationResponse>, AppError> {
    let target = OrganizationId::new(&id);
    require_permission(
        &state,
        &user_id,
        policy::organization::update(target.clone()),
    )
    .await?;

    let name = body
        .name
        .as_deref()
        .map(OrganizationName::new)
        .transpose()
        .map_err(AppError::from)?;
    let description = body
        .description
        .as_deref()
        .map(|d| OrganizationDescription::new(d).map(Some))
        .transpose()
        .map_err(AppError::from)?;

    let org = state.org_uc.update(&target, name, description).await?;
    Ok(Json(OrganizationResponse::from(&org)))
}

#[utoipa::path(
    post, path = "/api/v1/organizations/{id}/toggle-active",
    tag = "Organizations",
    params(("id" = String, Path, description = "Organization ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Active status toggled"),
        (status = 404, description = "Not found", body = ErrorBody),
    )
)]
pub(crate) async fn toggle_active(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
) -> Result<Json<()>, AppError> {
    let target = OrganizationId::new(&id);
    require_permission(
        &state,
        &user_id,
        policy::organization::toggle_active(target.clone()),
    )
    .await?;

    state.org_uc.toggle_active(&target).await?;
    Ok(Json(()))
}

#[utoipa::path(
    delete, path = "/api/v1/organizations/{id}",
    tag = "Organizations",
    params(("id" = String, Path, description = "Organization ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Organization deleted"),
        (status = 404, description = "Not found", body = ErrorBody),
    )
)]
pub(crate) async fn delete_one(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
) -> Result<Json<()>, AppError> {
    let target = OrganizationId::new(&id);
    require_permission(
        &state,
        &user_id,
        policy::organization::delete(target.clone()),
    )
    .await?;

    state.org_uc.delete(&target).await?;
    Ok(Json(()))
}

#[utoipa::path(
    get, path = "/api/v1/organizations",
    tag = "Organizations",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Items per page"),
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of organizations", body = ListOrganizationsResponse),
    )
)]
pub(crate) async fn list(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ListOrganizationsResponse>, AppError> {
    require_permission(&state, &user_id, policy::organization::list()).await?;

    let params = to_pagination_params(&pagination);
    let result = state.org_uc.list(params.as_ref()).await?;
    let (orgs, metadata) = result.into_parts();

    Ok(Json(ListOrganizationsResponse {
        organizations: orgs.iter().map(OrganizationResponse::from).collect(),
        pagination: PaginationMeta::from(&metadata),
    }))
}

#[utoipa::path(
    get, path = "/api/v1/organizations/{id}/users",
    tag = "Organizations",
    params(
        ("id" = String, Path, description = "Organization ID"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Items per page"),
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Organization members", body = ListMembersResponse),
    )
)]
pub(crate) async fn list_users(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ListMembersResponse>, AppError> {
    let target = OrganizationId::new(&id);
    require_permission(
        &state,
        &user_id,
        policy::organization::list_users(target.clone()),
    )
    .await?;

    let params = to_pagination_params(&pagination);
    let (users, metadata) = state.org_uc.list_users(&target, params.as_ref()).await?;

    Ok(Json(ListMembersResponse {
        members: users
            .iter()
            .map(|u| MemberResponse {
                user_id: u.id().to_string(),
                username: u.username().to_string(),
            })
            .collect(),
        pagination: PaginationMeta::from(&metadata),
    }))
}

#[utoipa::path(
    post, path = "/api/v1/organizations/{id}/users",
    tag = "Organizations",
    params(("id" = String, Path, description = "Organization ID")),
    request_body = AddUserRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "User added to organization"),
        (status = 404, description = "Not found", body = ErrorBody),
    )
)]
pub(crate) async fn add_user(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
    Json(body): Json<AddUserRequest>,
) -> Result<Json<()>, AppError> {
    let target = OrganizationId::new(&id);
    require_permission(
        &state,
        &user_id,
        policy::organization::add_user_to_organization(target.clone()),
    )
    .await?;

    let target_user = UserId::new(&body.user_id);
    state.org_uc.add_user(&target_user, &target).await?;
    Ok(Json(()))
}

#[utoipa::path(
    delete, path = "/api/v1/organizations/{id}/users/{user_id}",
    tag = "Organizations",
    params(
        ("id" = String, Path, description = "Organization ID"),
        ("user_id" = String, Path, description = "User ID to remove"),
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "User removed from organization"),
        (status = 404, description = "Not found", body = ErrorBody),
    )
)]
pub(crate) async fn remove_user(
    State(state): State<AppState>,
    Auth(caller_id): Auth,
    Path((id, target_user_id)): Path<(String, String)>,
) -> Result<Json<()>, AppError> {
    let target = OrganizationId::new(&id);
    require_permission(
        &state,
        &caller_id,
        policy::organization::remove_user_from_organization(target.clone()),
    )
    .await?;

    let target_user = UserId::new(&target_user_id);
    state.org_uc.remove_user(&target_user, &target).await?;
    Ok(Json(()))
}

#[utoipa::path(
    get, path = "/api/v1/organizations/{id}/pipelines",
    tag = "Organizations",
    params(
        ("id" = String, Path, description = "Organization ID"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Items per page"),
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Organization pipelines", body = ListPipelinesResponse),
    )
)]
pub(crate) async fn list_pipelines(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ListPipelinesResponse>, AppError> {
    let target = OrganizationId::new(&id);
    require_permission(
        &state,
        &user_id,
        policy::pipeline::list_by_organization(target.clone()),
    )
    .await?;

    let params = to_pagination_params(&pagination);
    let result = state
        .pipeline_uc
        .list_by_organization(&target, params.as_ref())
        .await?;
    let (pipelines, metadata) = result.into_parts();

    Ok(Json(ListPipelinesResponse {
        pipelines: pipelines
            .iter()
            .map(PipelineSummaryResponse::from)
            .collect(),
        pagination: PaginationMeta::from(&metadata),
    }))
}

#[utoipa::path(
    get, path = "/api/v1/organizations/{id}/jobs",
    tag = "Organizations",
    params(
        ("id" = String, Path, description = "Organization ID"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Items per page"),
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Organization jobs", body = ListJobsResponse),
    )
)]
pub(crate) async fn list_jobs(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ListJobsResponse>, AppError> {
    let target = OrganizationId::new(&id);
    require_permission(
        &state,
        &user_id,
        policy::job::list_by_organization(target.clone()),
    )
    .await?;

    let params = to_pagination_params(&pagination);
    let result = state
        .job_uc
        .list_by_organization(&target, params.as_ref())
        .await?;
    let (jobs, metadata) = result.into_parts();

    Ok(Json(ListJobsResponse {
        jobs: jobs.iter().map(JobResponse::from).collect(),
        pagination: PaginationMeta::from(&metadata),
    }))
}
