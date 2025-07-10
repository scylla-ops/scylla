use crate::api::v1::common::base::{BaseRepository, Repository};
use crate::api::v1::models::entities::Entity; // Replace with your actual model
use crate::api::v1::modules::template::dto::{NewEntity, UpdateEntity};
use crate::database::DieselPool;
use anyhow::{Context, Result};
use diesel::prelude::*;
use tracing::debug;

// Entity repository
// Replace "Entity" with your entity name (e.g., "User", "Team", "Product")
pub struct EntityRepository {
    base: BaseRepository,
}

// Repository trait for entity operations
// Replace "Entity" with your entity name
pub trait EntityRepositoryTrait {
    fn new(pool: DieselPool) -> Self;
    async fn create_entity(&self, new_entity: NewEntity) -> Result<usize>;
    async fn get_entity_by_uuid(&self, entity_uuid: uuid::Uuid) -> Result<Option<Entity>>;
    async fn get_all_entities(&self) -> Result<Vec<Entity>>;
    async fn update_entity_by_uuid(
        &self,
        entity_uuid: uuid::Uuid,
        updated_entity: UpdateEntity,
    ) -> Result<usize>;
    async fn delete_entity_by_uuid(&self, entity_uuid: uuid::Uuid) -> Result<usize>;
}

impl EntityRepositoryTrait for EntityRepository {
    fn new(pool: DieselPool) -> Self {
        Self {
            base: BaseRepository::new(pool),
        }
    }

    /// Creates a new entity in the database.
    /// # Arguments
    /// * `new_entity` - The new entity data to be inserted.
    /// # Returns
    /// * `Result<usize>` - The number of rows inserted, or an error if the operation fails.
    async fn create_entity(&self, new_entity: NewEntity) -> Result<usize> {
        use crate::database::schema::entities::dsl::*; // Replace with your actual table

        let mut conn = Repository::get_connection(self)?;

        let inserted_count = diesel::insert_into(entities) // Replace with your actual table
            .values(&new_entity)
            .execute(&mut conn)
            .context("Failed to insert new entity")?;

        debug!("Inserted {} new entity(ies)", inserted_count);
        Ok(inserted_count)
    }

    /// Fetches an entity by its UUID.
    /// # Arguments
    /// * `entity_uuid` - The UUID of the entity to be fetched.
    /// # Returns
    /// * `Result<Option<Entity>>` - The entity if found, or None if not found, or an error if the operation fails.
    async fn get_entity_by_uuid(&self, entity_uuid: uuid::Uuid) -> Result<Option<Entity>> {
        use crate::database::schema::entities::dsl::*; // Replace with your actual table

        let mut conn = Repository::get_connection(self)?;

        let entity = entities // Replace with your actual table
            .filter(id.eq(entity_uuid))
            .first::<Entity>(&mut conn)
            .optional()
            .context("Failed to fetch entity by UUID")?;

        debug!("Fetched entity: {:?}", entity);
        Ok(entity)
    }

    /// Fetches all entities from the database.
    /// # Returns
    /// * `Result<Vec<Entity>>` - A vector of entities, or an error if the operation fails.
    async fn get_all_entities(&self) -> Result<Vec<Entity>> {
        use crate::database::schema::entities::dsl::*; // Replace with your actual table

        let mut conn = Repository::get_connection(self)?;

        let entities_list = entities // Replace with your actual table
            .load::<Entity>(&mut conn)
            .context("Failed to fetch all entities")?;

        debug!("Fetched {} entities", entities_list.len());
        Ok(entities_list)
    }

    /// Updates an entity by its UUID.
    /// # Arguments
    /// * `entity_uuid` - The UUID of the entity to be updated.
    /// * `updated_entity` - The updated entity data.
    /// # Returns
    /// * `Result<usize>` - The number of rows updated, or an error if the operation fails.
    async fn update_entity_by_uuid(
        &self,
        entity_uuid: uuid::Uuid,
        updated_entity: UpdateEntity,
    ) -> Result<usize> {
        use crate::database::schema::entities::dsl::*; // Replace with your actual table

        let mut conn = Repository::get_connection(self)?;

        let updated_count = diesel::update(entities.filter(id.eq(entity_uuid))) // Replace with your actual table
            .set(updated_entity)
            .execute(&mut conn)
            .context("Failed to update entity by UUID")?;

        debug!("Updated {} entity(ies)", updated_count);
        Ok(updated_count)
    }

    /// Deletes an entity by its UUID.
    /// # Arguments
    /// * `entity_uuid` - The UUID of the entity to be deleted.
    /// # Returns
    /// * `Result<usize>` - The number of rows deleted, or an error if the operation fails.
    async fn delete_entity_by_uuid(&self, entity_uuid: uuid::Uuid) -> Result<usize> {
        use crate::database::schema::entities::dsl::*; // Replace with your actual table

        let mut conn = Repository::get_connection(self)?;

        let deleted_count = diesel::delete(entities.filter(id.eq(entity_uuid))) // Replace with your actual table
            .execute(&mut conn)
            .context("Failed to delete entity by UUID")?;

        debug!("Deleted {} entity(ies)", deleted_count);
        Ok(deleted_count)
    }
}

// Implement Repository trait for EntityRepository by delegating to the base repository
impl Repository for EntityRepository {
    fn get_pool(&self) -> &DieselPool {
        self.base.get_pool()
    }
}
