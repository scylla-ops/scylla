use crate::api::v1::common::extractors::validated_json::ValidatedJson;
use crate::api::v1::common::responses::helper::ApiResponse;
use crate::api::v1::modules::template::dto::{NewEntityRequest, UpdateEntityRequest};
use crate::api::v1::modules::template::repository::EntityRepository;
use crate::api::v1::modules::template::service::{EntityService, EntityServiceTrait};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use std::sync::Arc;
use uuid::Uuid;

// Replace "Entity" with your entity name (e.g., "User", "Team", "Product")
type EntityState = State<Arc<EntityService<EntityRepository>>>;

pub struct EntityController {}

impl EntityController {
    // Create a new entity
    pub async fn create_entity(
        State(service): EntityState,
        ValidatedJson(req): ValidatedJson<NewEntityRequest>,
    ) -> impl IntoResponse {
        match service.create_entity(req).await {
            Ok(_) => {
                ApiResponse::success(axum::http::StatusCode::CREATED, "Entity created successfully")
            }
            Err(e) => {
                tracing::error!("Failed to create entity: {}", e);
                ApiResponse::error(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create entity: {e}"),
                )
            }
        }
    }

    // Get entity by ID
    pub async fn get_entity_by_id(
        State(service): EntityState,
        entity_uuid: Path<Uuid>,
    ) -> impl IntoResponse {
        match service.get_entity_by_id(*entity_uuid).await {
            Ok(Some(entity)) => ApiResponse::success(axum::http::StatusCode::OK, entity),
            Ok(None) => ApiResponse::error(
                axum::http::StatusCode::NOT_FOUND,
                "Entity not found".to_string(),
            ),
            Err(e) => {
                tracing::error!("Failed to fetch entity: {}", e);
                ApiResponse::error(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to fetch entity: {e}"),
                )
            }
        }
    }

    // Get all entities
    pub async fn get_all_entities(State(service): EntityState) -> impl IntoResponse {
        match service.get_all_entities().await {
            Ok(entities) => ApiResponse::success(axum::http::StatusCode::OK, entities),
            Err(e) => {
                tracing::error!("Failed to fetch entities: {}", e);
                ApiResponse::error(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to fetch entities: {e}"),
                )
            }
        }
    }

    // Update entity by ID
    pub async fn update_entity_by_id(
        State(service): EntityState,
        entity_uuid: Path<Uuid>,
        ValidatedJson(req): ValidatedJson<UpdateEntityRequest>,
    ) -> impl IntoResponse {
        match service.update_entity_by_id(*entity_uuid, req).await {
            Ok(_) => ApiResponse::success(axum::http::StatusCode::OK, "Entity updated successfully"),
            Err(e) => {
                tracing::error!("Failed to update entity: {}", e);
                ApiResponse::error(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to update entity: {e}"),
                )
            }
        }
    }

    // Delete entity by ID
    pub async fn delete_entity_by_id(
        State(service): EntityState,
        entity_uuid: Path<Uuid>,
    ) -> impl IntoResponse {
        match service.delete_entity_by_id(*entity_uuid).await {
            Ok(_) => {
                ApiResponse::success(axum::http::StatusCode::OK, "Entity deleted successfully")
            }
            Err(e) => {
                tracing::error!("Failed to delete entity: {}", e);
                ApiResponse::error(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to delete entity: {e}"),
                )
            }
        }
    }
}
