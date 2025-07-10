use crate::api::v1::common::extractors::validated_json::ValidatedJson;
use crate::api::v1::common::responses::helper::ApiResponse;
use crate::api::v1::modules::teams::dto::{NewTeamRequest, UpdateTeamRequest};
use crate::api::v1::modules::teams::repository::TeamRepository;
use crate::api::v1::modules::teams::service::{TeamService, TeamServiceTrait};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use std::sync::Arc;
use uuid::Uuid;

type TeamState = State<Arc<TeamService<TeamRepository>>>;

pub struct TeamController {}

impl TeamController {
    pub async fn create_team(
        State(service): TeamState,
        ValidatedJson(req): ValidatedJson<NewTeamRequest>,
    ) -> impl IntoResponse {
        match service.create_team(req).await {
            Ok(team_uuid) => ApiResponse::success(
                axum::http::StatusCode::CREATED,
                format!("Team created successfully with UUID: {team_uuid}"),
            ),
            Err(e) => {
                tracing::error!("Failed to create team: {}", e);
                ApiResponse::error(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create team: {e}"),
                )
            }
        }
    }

    pub async fn get_team_by_id(
        State(service): TeamState,
        Path(team_id): Path<Uuid>,
    ) -> impl IntoResponse {
        // Logique pour récupérer une équipe par ID
        match service.get_team_by_id(team_id).await {
            Ok(Some(team)) => ApiResponse::success(axum::http::StatusCode::OK, team),
            Ok(None) => ApiResponse::error(
                axum::http::StatusCode::NOT_FOUND,
                "Team not found".to_string(),
            ),
            Err(e) => {
                tracing::error!("Failed to fetch team: {}", e);
                ApiResponse::error(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to fetch team: {e}"),
                )
            }
        }
    }

    pub async fn get_all_teams(State(service): TeamState) -> impl IntoResponse {
        // Logique pour récupérer toutes les équipes
        match service.get_all_teams().await {
            Ok(teams) => ApiResponse::success(axum::http::StatusCode::OK, teams),
            Err(e) => {
                tracing::error!("Failed to fetch teams: {}", e);
                ApiResponse::error(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to fetch teams: {e}"),
                )
            }
        }
    }

    pub async fn update_team_by_id(
        State(service): TeamState,
        Path(team_id): Path<Uuid>,
        ValidatedJson(req): ValidatedJson<UpdateTeamRequest>,
    ) -> impl IntoResponse {
        match service.update_team_by_id(team_id, req).await {
            Ok(_) => ApiResponse::success(
                axum::http::StatusCode::OK,
                "Team updated successfully".to_string(),
            ),
            Err(e) => {
                tracing::error!("Failed to update team: {}", e);
                ApiResponse::error(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to update team: {e}"),
                )
            }
        }
    }

    pub async fn delete_team_by_id(
        State(service): TeamState,
        Path(team_id): Path<Uuid>,
    ) -> impl IntoResponse {
        match service.delete_team_by_id(team_id).await {
            Ok(_) => ApiResponse::success(axum::http::StatusCode::NO_CONTENT, "".to_string()),
            Err(e) => {
                tracing::error!("Failed to delete team: {}", e);
                ApiResponse::error(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to delete team: {e}"),
                )
            }
        }
    }
}
