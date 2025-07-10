use crate::api::v1::common::extractors::validated_json::ValidatedJson;
use crate::api::v1::common::responses::helper::ApiResponse;
use crate::api::v1::modules::teams::dto::NewTeamRequest;
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
        unimplemented!();
        "!!!!"
    }

    pub async fn update_team(
        State(service): TeamState,
        Path(team_id): Path<String>,
        // Json(req): Json<UpdateTeamRequest>,
    ) -> impl IntoResponse {
        // Logique pour mettre à jour une équipe
        unimplemented!();
        "!!!!"
    }

    pub async fn delete_team(
        State(service): TeamState,
        Path(team_id): Path<String>,
    ) -> impl IntoResponse {
        // Logique pour supprimer une équipe
        unimplemented!();
        "!!!!"
    }
}
