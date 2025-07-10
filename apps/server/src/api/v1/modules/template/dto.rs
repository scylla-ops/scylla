use crate::api::v1::models::entities::Entity; // Replace with your actual model
use diesel::{AsChangeset, Insertable};
use serde::{Deserialize, Serialize};
use validator::Validate;

// Constants for validation
// Customize these constants based on your entity's requirements
const NAME_MIN_LENGTH: u64 = 1;
const NAME_MAX_LENGTH: u64 = 255;

// DB only - Used for database insertion
// Replace "entities" with your actual table name in the diesel attribute
// Add or remove fields based on your entity's requirements
#[derive(Insertable, Deserialize, Validate)]
#[diesel(table_name = crate::database::schema::entities)] // Replace with your actual table
pub struct NewEntity {
    pub name: String,
    // Add other fields as needed
}

// Request DTO for creating a new entity
// Add or remove fields based on your entity's requirements
#[derive(Deserialize, Validate)]
pub struct NewEntityRequest {
    #[validate(length(
        min = NAME_MIN_LENGTH,
        max = NAME_MAX_LENGTH,
        message = "Name must be between 1 and 255 characters"
    ))]
    pub name: String,
    // Add other fields as needed
}

// Response DTO for entity data
// Add or remove fields based on your entity's requirements
#[derive(Serialize)]
pub struct EntityResponse {
    pub uuid: uuid::Uuid,
    pub name: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    // Add other fields as needed
}

// Conversion from Entity model to EntityResponse
// Update this implementation based on your entity's fields
impl From<Entity> for EntityResponse {
    fn from(entity: Entity) -> Self {
        Self {
            uuid: entity.id,
            name: entity.name,
            is_active: entity.is_active,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            // Map other fields as needed
        }
    }
}

// Conversion from NewEntityRequest to NewEntity
// Update this implementation based on your entity's fields
impl TryFrom<NewEntityRequest> for NewEntity {
    type Error = anyhow::Error;

    fn try_from(req: NewEntityRequest) -> anyhow::Result<Self> {
        // Perform any necessary transformations or validations
        Ok(Self {
            name: req.name,
            // Map other fields as needed
        })
    }
}

// Request DTO for updating an entity
// Add or remove fields based on your entity's requirements
#[derive(Deserialize, Validate)]
pub struct UpdateEntityRequest {
    #[validate(length(
        min = NAME_MIN_LENGTH,
        max = NAME_MAX_LENGTH,
        message = "Name must be between 1 and 255 characters"
    ))]
    pub name: Option<String>,
    pub is_active: Option<bool>,
    // Add other fields as needed
}

// DB only - Used for database updates
// Replace "entities" with your actual table name in the diesel attribute
// Add or remove fields based on your entity's requirements
#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::database::schema::entities)] // Replace with your actual table
pub struct UpdateEntity {
    pub name: Option<String>,
    pub is_active: Option<bool>,
    pub updated_at: chrono::NaiveDateTime,
    // Add other fields as needed
}

// Default implementation for UpdateEntity
// Update this implementation based on your entity's fields
impl Default for UpdateEntity {
    fn default() -> Self {
        Self {
            name: None,
            is_active: None,
            updated_at: chrono::Utc::now().naive_utc(),
            // Set defaults for other fields as needed
        }
    }
}

// Conversion from UpdateEntityRequest to UpdateEntity
// Update this implementation based on your entity's fields
impl TryFrom<UpdateEntityRequest> for UpdateEntity {
    type Error = anyhow::Error;

    fn try_from(req: UpdateEntityRequest) -> anyhow::Result<Self> {
        Ok(Self {
            name: req.name,
            is_active: req.is_active,
            updated_at: chrono::Utc::now().naive_utc(),
            // Map other fields as needed
        })
    }
}
