use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use scylla_core::domain::entities::{PipelineId, PipelineNode, ProjectId};
use scylla_core::domain::value_objects::permission::policy;
use scylla_core::domain::value_objects::pipeline::{NodeId, PipelineName};

use crate::rest::AppState;
use crate::rest::error::AppError;
use crate::rest::extract::Auth;
use crate::rest::response::*;

use super::users::{require_permission, to_pagination_params};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create).get(list))
        .route("/{id}", get(get_one).put(update).delete(delete_one))
        .route("/{id}/jobs", get(list_jobs))
}

#[utoipa::path(
    post, path = "/api/v1/pipelines",
    tag = "Pipelines",
    request_body = CreatePipelineRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Pipeline created", body = PipelineResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
    )
)]
pub(crate) async fn create(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Json(body): Json<CreatePipelineRequest>,
) -> Result<Json<PipelineResponse>, AppError> {
    let project_id = ProjectId::new(&body.project_id);
    require_permission(
        &state,
        &user_id,
        policy::pipeline::create(project_id.clone()),
    )
    .await?;

    let name = PipelineName::new(&body.name).map_err(AppError::from)?;
    let nodes = parse_nodes(&body.nodes)?;

    let pipeline = state.pipeline_uc.create(name, project_id, nodes).await?;
    Ok(Json(PipelineResponse::from(&pipeline)))
}

#[utoipa::path(
    get, path = "/api/v1/pipelines/{id}",
    tag = "Pipelines",
    params(("id" = String, Path, description = "Pipeline ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Pipeline found", body = PipelineResponse),
        (status = 404, description = "Not found", body = ErrorBody),
    )
)]
pub(crate) async fn get_one(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
) -> Result<Json<PipelineResponse>, AppError> {
    let target = PipelineId::new(&id);
    require_permission(&state, &user_id, policy::pipeline::get(target.clone())).await?;

    let pipeline = state.pipeline_uc.get(&target).await?;
    Ok(Json(PipelineResponse::from(&pipeline)))
}

#[utoipa::path(
    put, path = "/api/v1/pipelines/{id}",
    tag = "Pipelines",
    params(("id" = String, Path, description = "Pipeline ID")),
    request_body = UpdatePipelineRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Pipeline updated", body = PipelineResponse),
        (status = 404, description = "Not found", body = ErrorBody),
    )
)]
pub(crate) async fn update(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
    Json(body): Json<UpdatePipelineRequest>,
) -> Result<Json<PipelineResponse>, AppError> {
    let target = PipelineId::new(&id);
    require_permission(&state, &user_id, policy::pipeline::update(target.clone())).await?;

    let name = body
        .name
        .as_deref()
        .map(PipelineName::new)
        .transpose()
        .map_err(AppError::from)?;
    let nodes = body.nodes.as_ref().map(|n| parse_nodes(n)).transpose()?;

    let pipeline = state.pipeline_uc.update(&target, name, nodes).await?;
    Ok(Json(PipelineResponse::from(&pipeline)))
}

#[utoipa::path(
    delete, path = "/api/v1/pipelines/{id}",
    tag = "Pipelines",
    params(("id" = String, Path, description = "Pipeline ID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Pipeline deleted"),
        (status = 404, description = "Not found", body = ErrorBody),
    )
)]
pub(crate) async fn delete_one(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
) -> Result<Json<()>, AppError> {
    let target = PipelineId::new(&id);
    require_permission(&state, &user_id, policy::pipeline::delete(target.clone())).await?;

    state.pipeline_uc.delete(&target).await?;
    Ok(Json(()))
}

#[utoipa::path(
    get, path = "/api/v1/pipelines",
    tag = "Pipelines",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Items per page"),
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of pipelines", body = ListPipelinesResponse),
    )
)]
pub(crate) async fn list(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ListPipelinesResponse>, AppError> {
    require_permission(&state, &user_id, policy::pipeline::list()).await?;

    let params = to_pagination_params(&pagination);
    let result = state.pipeline_uc.list(params.as_ref()).await?;
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
    get, path = "/api/v1/pipelines/{id}/jobs",
    tag = "Pipelines",
    params(
        ("id" = String, Path, description = "Pipeline ID"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Items per page"),
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Pipeline jobs", body = ListJobsResponse),
    )
)]
pub(crate) async fn list_jobs(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Path(id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ListJobsResponse>, AppError> {
    let target = PipelineId::new(&id);
    require_permission(&state, &user_id, policy::job::list_by_pipeline()).await?;

    let params = to_pagination_params(&pagination);
    let result = state
        .job_uc
        .list_by_pipeline(&target, params.as_ref())
        .await?;
    let (jobs, metadata) = result.into_parts();

    Ok(Json(ListJobsResponse {
        jobs: jobs.iter().map(JobResponse::from).collect(),
        pagination: PaginationMeta::from(&metadata),
    }))
}

fn parse_nodes(nodes: &[PipelineNodeRequest]) -> Result<Vec<PipelineNode>, AppError> {
    nodes
        .iter()
        .map(|n| {
            let id = NodeId::new(&n.node_id).map_err(AppError::from)?;
            let deps = n
                .deps
                .iter()
                .map(|d| NodeId::new(d).map_err(AppError::from))
                .collect::<Result<Vec<_>, _>>()?;
            PipelineNode::new(id, deps, n.command.clone(), n.args.clone()).map_err(AppError::from)
        })
        .collect()
}
