use crate::api::v1::common::base::Repository;
use crate::api::v1::modules::template::dto::{EntityResponse, NewEntityRequest, UpdateEntityRequest};
use crate::api::v1::modules::template::repository::{EntityRepository, EntityRepositoryTrait};
use anyhow::Result;
use uuid::Uuid;

// Entity service for handling entity-related business logic
// Replace "Entity" with your entity name (e.g., "User", "Team", "Product")
pub struct EntityService<R: Repository + EntityRepositoryTrait> {
    repository: R,
}

// Service trait for entity operations
// Replace "Entity" with your entity name
pub trait EntityServiceTrait<R: Repository + EntityRepositoryTrait> {
    fn new(repository: R) -> Self;
    async fn create_entity(&self, req: NewEntityRequest) -> Result<usize>;
    async fn get_entity_by_id(&self, entity_uuid: Uuid) -> Result<Option<EntityResponse>>;
    async fn get_all_entities(&self) -> Result<Vec<EntityResponse>>;
    async fn update_entity_by_id(&self, entity_uuid: Uuid, req: UpdateEntityRequest) -> Result<()>;
    async fn delete_entity_by_id(&self, entity_uuid: Uuid) -> Result<()>;
}

impl<R: Repository + EntityRepositoryTrait> EntityServiceTrait<R> for EntityService<R> {
    fn new(repository: R) -> Self {
        Self { repository }
    }

    // Create a new entity
    async fn create_entity(&self, req: NewEntityRequest) -> Result<usize> {
        self.repository.create_entity(req.try_into()?).await
    }

    // Get entity by ID
    async fn get_entity_by_id(&self, entity_uuid: Uuid) -> Result<Option<EntityResponse>> {
        Ok(self
            .repository
            .get_entity_by_uuid(entity_uuid)
            .await?
            .map(EntityResponse::from))
    }

    // Get all entities
    async fn get_all_entities(&self) -> Result<Vec<EntityResponse>> {
        let entities = self.repository.get_all_entities().await?;
        Ok(entities.into_iter().map(EntityResponse::from).collect())
    }

    // Update entity by ID
    async fn update_entity_by_id(&self, entity_uuid: Uuid, req: UpdateEntityRequest) -> Result<()> {
        self.repository
            .update_entity_by_uuid(entity_uuid, req.try_into()?)
            .await?;
        Ok(())
    }

    // Delete entity by ID
    async fn delete_entity_by_id(&self, entity_uuid: Uuid) -> Result<()> {
        self.repository.delete_entity_by_uuid(entity_uuid).await?;
        Ok(())
    }
}
