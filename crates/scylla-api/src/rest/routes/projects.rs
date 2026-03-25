use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use scylla_core::domain::entities::{OrganizationId, ProjectId, UserId};
use scylla_core::domain::value_objects::permission::policy;
use scylla_core::domain::value_objects::project::{ProjectDescription, ProjectName};

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
    post, path = "/api/v1/projects",
    tag = "Projects",
    request_body = CreateProjectRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Project created", body = ProjectResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
    )
)]
pub(crate) async fn create(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Json(body): Json<CreateProjectRequest>,
) -> Result<Json<ProjectResponse>, AppError> {
    let org_id = OrganizationId::new(&body.organization_id);
    require_permission(&state, &user_id, policy::project::create(org_id.clone())).await?;

    let name = ProjectName::new(&body.name).map_err(AppError::from)?;
    let description = body
        .description
        .as_deref()
        .map(ProjectDescription::new)
        .transpose()
        .map_err(AppError::from)?;

    let project = state.project_uc.create(name, description, org_id).await?;
    Ok(Json(ProjectResponse::from(&project)))
}

#[utoipa::path(
    get, path = "/api/v1/projects/{id}",
    tag = "Projects",
    params(("id" = String, Path, description = "Project ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Project found", body = ProjectResponse),
        (status = 404, description = "Not found", body = ErrorBody),
    )
)]
pub(crate) async fn get_one(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
) -> Result<Json<ProjectResponse>, AppError> {
    let target = ProjectId::new(&id);
    require_permission(&state, &user_id, policy::project::get(target.clone())).await?;

    let project = state.project_uc.get(&target).await?;
    Ok(Json(ProjectResponse::from(&project)))
}

#[utoipa::path(
    put, path = "/api/v1/projects/{id}",
    tag = "Projects",
    params(("id" = String, Path, description = "Project ID")),
    request_body = UpdateProjectRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Project updated", body = ProjectResponse),
        (status = 404, description = "Not found", body = ErrorBody),
    )
)]
pub(crate) async fn update(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
    Json(body): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectResponse>, AppError> {
    let target = ProjectId::new(&id);
    require_permission(&state, &user_id, policy::project::update(target.clone())).await?;

    let name = body
        .name
        .as_deref()
        .map(ProjectName::new)
        .transpose()
        .map_err(AppError::from)?;
    let description = body
        .description
        .as_deref()
        .map(|d| ProjectDescription::new(d).map(Some))
        .transpose()
        .map_err(AppError::from)?;

    let project = state.project_uc.update(&target, name, description).await?;
    Ok(Json(ProjectResponse::from(&project)))
}

#[utoipa::path(
    post, path = "/api/v1/projects/{id}/toggle-active",
    tag = "Projects",
    params(("id" = String, Path, description = "Project ID")),
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
    let target = ProjectId::new(&id);
    require_permission(
        &state,
        &user_id,
        policy::project::toggle_active(target.clone()),
    )
    .await?;

    state.project_uc.toggle_active(&target).await?;
    Ok(Json(()))
}

#[utoipa::path(
    delete, path = "/api/v1/projects/{id}",
    tag = "Projects",
    params(("id" = String, Path, description = "Project ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Project deleted"),
        (status = 404, description = "Not found", body = ErrorBody),
    )
)]
pub(crate) async fn delete_one(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
) -> Result<Json<()>, AppError> {
    let target = ProjectId::new(&id);
    require_permission(&state, &user_id, policy::project::delete(target.clone())).await?;

    state.project_uc.delete(&target).await?;
    Ok(Json(()))
}

#[utoipa::path(
    get, path = "/api/v1/projects",
    tag = "Projects",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Items per page"),
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of projects", body = ListProjectsResponse),
    )
)]
pub(crate) async fn list(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ListProjectsResponse>, AppError> {
    require_permission(&state, &user_id, policy::project::list()).await?;

    let params = to_pagination_params(&pagination);
    let result = state.project_uc.list(params.as_ref()).await?;
    let (projects, metadata) = result.into_parts();

    Ok(Json(ListProjectsResponse {
        projects: projects.iter().map(ProjectResponse::from).collect(),
        pagination: PaginationMeta::from(&metadata),
    }))
}

#[utoipa::path(
    get, path = "/api/v1/projects/{id}/users",
    tag = "Projects",
    params(
        ("id" = String, Path, description = "Project ID"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Items per page"),
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Project members", body = ListMembersResponse),
    )
)]
pub(crate) async fn list_users(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ListMembersResponse>, AppError> {
    let target = ProjectId::new(&id);
    require_permission(
        &state,
        &user_id,
        policy::project::list_users(target.clone()),
    )
    .await?;

    let params = to_pagination_params(&pagination);
    let (users, metadata) = state
        .project_uc
        .list_users(&target, params.as_ref())
        .await?;

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
    post, path = "/api/v1/projects/{id}/users",
    tag = "Projects",
    params(("id" = String, Path, description = "Project ID")),
    request_body = AddUserRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "User added to project"),
        (status = 404, description = "Not found", body = ErrorBody),
    )
)]
pub(crate) async fn add_user(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
    Json(body): Json<AddUserRequest>,
) -> Result<Json<()>, AppError> {
    let target = ProjectId::new(&id);
    require_permission(
        &state,
        &user_id,
        policy::project::add_user_to_project(target.clone()),
    )
    .await?;

    let target_user = UserId::new(&body.user_id);
    state.project_uc.add_user(&target_user, &target).await?;
    Ok(Json(()))
}

#[utoipa::path(
    delete, path = "/api/v1/projects/{id}/users/{user_id}",
    tag = "Projects",
    params(
        ("id" = String, Path, description = "Project ID"),
        ("user_id" = String, Path, description = "User ID to remove"),
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "User removed from project"),
        (status = 404, description = "Not found", body = ErrorBody),
    )
)]
pub(crate) async fn remove_user(
    State(state): State<AppState>,
    Auth(caller_id): Auth,
    Path((id, target_user_id)): Path<(String, String)>,
) -> Result<Json<()>, AppError> {
    let target = ProjectId::new(&id);
    require_permission(
        &state,
        &caller_id,
        policy::project::remove_user_from_project(target.clone()),
    )
    .await?;

    let target_user = UserId::new(&target_user_id);
    state.project_uc.remove_user(&target_user, &target).await?;
    Ok(Json(()))
}

#[utoipa::path(
    get, path = "/api/v1/projects/{id}/pipelines",
    tag = "Projects",
    params(
        ("id" = String, Path, description = "Project ID"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Items per page"),
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Project pipelines", body = ListPipelinesResponse),
    )
)]
pub(crate) async fn list_pipelines(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ListPipelinesResponse>, AppError> {
    let target = ProjectId::new(&id);
    require_permission(
        &state,
        &user_id,
        policy::pipeline::list_by_project(target.clone()),
    )
    .await?;

    let params = to_pagination_params(&pagination);
    let result = state
        .pipeline_uc
        .list_by_project(&target, params.as_ref())
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
    get, path = "/api/v1/projects/{id}/jobs",
    tag = "Projects",
    params(
        ("id" = String, Path, description = "Project ID"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Items per page"),
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Project jobs", body = ListJobsResponse),
    )
)]
pub(crate) async fn list_jobs(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ListJobsResponse>, AppError> {
    let target = ProjectId::new(&id);
    require_permission(
        &state,
        &user_id,
        policy::job::list_by_project(target.clone()),
    )
    .await?;

    let params = to_pagination_params(&pagination);
    let result = state
        .job_uc
        .list_by_project(&target, params.as_ref())
        .await?;
    let (jobs, metadata) = result.into_parts();

    Ok(Json(ListJobsResponse {
        jobs: jobs.iter().map(JobResponse::from).collect(),
        pagination: PaginationMeta::from(&metadata),
    }))
}
