use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use scylla_core::domain::entities::JobId;
use scylla_core::domain::value_objects::permission::policy;

use crate::rest::AppState;
use crate::rest::error::AppError;
use crate::rest::extract::Auth;
use crate::rest::response::*;

use super::users::{require_permission, to_pagination_params};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/{id}", get(get_one).delete(delete_one))
}

#[utoipa::path(
    get, path = "/api/v1/jobs/{id}",
    tag = "Jobs",
    params(("id" = String, Path, description = "Job ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Job found", body = JobResponse),
        (status = 404, description = "Not found", body = ErrorBody),
    )
)]
pub(crate) async fn get_one(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
) -> Result<Json<JobResponse>, AppError> {
    let target = JobId::new(&id);
    require_permission(&state, &user_id, policy::job::get(target.clone())).await?;

    let job = state.job_uc.get(&target).await?;
    Ok(Json(JobResponse::from(&job)))
}

#[utoipa::path(
    delete, path = "/api/v1/jobs/{id}",
    tag = "Jobs",
    params(("id" = String, Path, description = "Job ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Job deleted"),
        (status = 404, description = "Not found", body = ErrorBody),
    )
)]
pub(crate) async fn delete_one(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
) -> Result<Json<()>, AppError> {
    let target = JobId::new(&id);
    require_permission(&state, &user_id, policy::job::delete(target.clone())).await?;

    state.job_uc.delete(&target).await?;
    Ok(Json(()))
}

#[utoipa::path(
    get, path = "/api/v1/jobs",
    tag = "Jobs",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Items per page"),
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of jobs", body = ListJobsResponse),
    )
)]
pub(crate) async fn list(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ListJobsResponse>, AppError> {
    require_permission(&state, &user_id, policy::job::list()).await?;

    let params = to_pagination_params(&pagination);
    let result = state.job_uc.list(params.as_ref()).await?;
    let (jobs, metadata) = result.into_parts();

    Ok(Json(ListJobsResponse {
        jobs: jobs.iter().map(JobResponse::from).collect(),
        pagination: PaginationMeta::from(&metadata),
    }))
}
