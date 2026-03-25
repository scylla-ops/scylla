use axum::extract::{Query, State};
use axum::routing::post;
use axum::{Json, Router};
use scylla_core::domain::entities::UserId;
use scylla_core::domain::value_objects::permission::policy::{self, GroupingPolicy, Policy};
use scylla_core::domain::value_objects::permission::{Act, Resource, Scope};
use scylla_core::domain::value_objects::role::name::RoleName;

use crate::rest::AppState;
use crate::rest::error::AppError;
use crate::rest::extract::Auth;
use crate::rest::response::*;

use super::users::require_permission;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/policies",
            post(add_policy).delete(remove_policy).get(list_policies),
        )
        .route(
            "/grouping-policies",
            post(add_grouping_policy)
                .delete(remove_grouping_policy)
                .get(list_grouping_policies),
        )
}

fn parse_scope(s: &str, scope_id: Option<&str>) -> Result<Scope, AppError> {
    let raw = match scope_id {
        Some(id) => format!("{s}/{id}"),
        None => s.to_string(),
    };
    raw.parse::<Scope>()
        .map_err(|_| AppError::validation(format!("Invalid scope: {raw}")))
}

fn parse_resource(s: &str, resource_id: Option<&str>) -> Result<Resource, AppError> {
    let raw = match resource_id {
        Some(id) => format!("{s}/{id}"),
        None => format!("{s}/*"),
    };
    raw.parse::<Resource>()
        .map_err(|_| AppError::validation(format!("Invalid resource: {raw}")))
}

fn parse_act(s: &str) -> Result<Act, AppError> {
    s.parse::<Act>()
        .map_err(|_| AppError::validation(format!("Invalid act: {s}")))
}

#[utoipa::path(
    post, path = "/api/v1/permissions/policies",
    tag = "Permissions",
    request_body = PolicyRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Policy added", body = BoolResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
    )
)]
pub(crate) async fn add_policy(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Json(body): Json<PolicyRequest>,
) -> Result<Json<BoolResponse>, AppError> {
    require_permission(&state, &user_id, policy::permission::manage()).await?;

    let subject = UserId::new(&body.subject);
    let scope = parse_scope(&body.scope, body.scope_id.as_deref())?;
    let resource = parse_resource(&body.resource, body.resource_id.as_deref())?;
    let act = parse_act(&body.act)?;
    let p = Policy::new(scope, resource, act);

    let added = state.permission_uc.add_policy(subject, p).await?;
    Ok(Json(BoolResponse { success: added }))
}

#[utoipa::path(
    delete, path = "/api/v1/permissions/policies",
    tag = "Permissions",
    request_body = PolicyRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Policy removed", body = BoolResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
    )
)]
pub(crate) async fn remove_policy(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Json(body): Json<PolicyRequest>,
) -> Result<Json<BoolResponse>, AppError> {
    require_permission(&state, &user_id, policy::permission::manage()).await?;

    let subject = UserId::new(&body.subject);
    let scope = parse_scope(&body.scope, body.scope_id.as_deref())?;
    let resource = parse_resource(&body.resource, body.resource_id.as_deref())?;
    let act = parse_act(&body.act)?;
    let p = Policy::new(scope, resource, act);

    let removed = state.permission_uc.remove_policy(subject, p).await?;
    Ok(Json(BoolResponse { success: removed }))
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct ListPoliciesQuery {
    pub subject: Option<String>,
}

#[utoipa::path(
    get, path = "/api/v1/permissions/policies",
    tag = "Permissions",
    params(ListPoliciesQuery),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of policies", body = ListPoliciesResponse),
    )
)]
pub(crate) async fn list_policies(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Query(query): Query<ListPoliciesQuery>,
) -> Result<Json<ListPoliciesResponse>, AppError> {
    require_permission(&state, &user_id, policy::permission::list()).await?;

    let rows = state
        .permission_uc
        .list_policies(query.subject.as_deref())
        .await?;

    let policies = rows
        .into_iter()
        .map(|(sub, p)| PolicyResponse {
            subject: sub,
            scope: p.scope.as_str(),
            resource: p.resource.as_str(),
            act: p.act.as_str().to_string(),
        })
        .collect();

    Ok(Json(ListPoliciesResponse { policies }))
}

#[utoipa::path(
    post, path = "/api/v1/permissions/grouping-policies",
    tag = "Permissions",
    request_body = GroupingPolicyRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Grouping policy added", body = BoolResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
    )
)]
pub(crate) async fn add_grouping_policy(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Json(body): Json<GroupingPolicyRequest>,
) -> Result<Json<BoolResponse>, AppError> {
    require_permission(&state, &user_id, policy::permission::manage()).await?;

    let subject = UserId::new(&body.subject);
    let scope = parse_scope(&body.scope, body.scope_id.as_deref())?;
    let role = RoleName::new(body.role).map_err(AppError::from)?;
    let gp = GroupingPolicy::new(role, scope);

    let added = state.permission_uc.add_grouping_policy(subject, gp).await?;
    Ok(Json(BoolResponse { success: added }))
}

#[utoipa::path(
    delete, path = "/api/v1/permissions/grouping-policies",
    tag = "Permissions",
    request_body = GroupingPolicyRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Grouping policy removed", body = BoolResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
    )
)]
pub(crate) async fn remove_grouping_policy(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Json(body): Json<GroupingPolicyRequest>,
) -> Result<Json<BoolResponse>, AppError> {
    require_permission(&state, &user_id, policy::permission::manage()).await?;

    let subject = UserId::new(&body.subject);
    let scope = parse_scope(&body.scope, body.scope_id.as_deref())?;
    let role = RoleName::new(body.role).map_err(AppError::from)?;
    let gp = GroupingPolicy::new(role, scope);

    let removed = state
        .permission_uc
        .remove_grouping_policy(subject, gp)
        .await?;
    Ok(Json(BoolResponse { success: removed }))
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct ListGroupingPoliciesQuery {
    pub subject: Option<String>,
}

#[utoipa::path(
    get, path = "/api/v1/permissions/grouping-policies",
    tag = "Permissions",
    params(ListGroupingPoliciesQuery),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of grouping policies", body = ListGroupingPoliciesResponse),
    )
)]
pub(crate) async fn list_grouping_policies(
    State(state): State<AppState>,
    Auth(user_id): Auth,
    Query(query): Query<ListGroupingPoliciesQuery>,
) -> Result<Json<ListGroupingPoliciesResponse>, AppError> {
    require_permission(&state, &user_id, policy::permission::list()).await?;

    let rows = state
        .permission_uc
        .list_grouping_policies(query.subject.as_deref())
        .await?;

    let policies = rows
        .into_iter()
        .map(|(sub, gp)| GroupingPolicyResponse {
            subject: sub,
            role: gp.role.into_string(),
            scope: gp.scope.as_str(),
        })
        .collect();

    Ok(Json(ListGroupingPoliciesResponse { policies }))
}
