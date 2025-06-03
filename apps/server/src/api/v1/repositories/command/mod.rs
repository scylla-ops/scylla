use anyhow::{Context, Result};
use diesel::prelude::*;
use tracing::info;
use crate::api::v1::models::Command;
use crate::api::v1::repositories::base::{BaseRepository, Repository};
use crate::database::DbPool;
use crate::database::schema::commands;

// Example command repository
pub struct CommandRepository {
    base: BaseRepository,
}

impl CommandRepository {
    pub fn new(pool: DbPool) -> Self {
        Self {
            base: BaseRepository::new(pool),
        }
    }

    // Example method to save a command to the database
    pub async fn save_command(&self, command_str: &str, args: &[String]) -> Result<String> {
        // Create a new Command model
        let command = Command::new(command_str.to_string(), args.to_vec());
        let command_id = command.id.clone();

        // Get a connection from the pool using the Repository trait method
        let mut conn = Repository::get_connection(self)?;

        // Insert the command into the database
        diesel::insert_into(commands::table)
            .values(&command)
            .execute(&mut conn)
            .context("Failed to insert command into database")?;

        info!("Command saved to database: {}", command_id);

        Ok(command_id)
    }

    // Example method to get a command from the database
    pub async fn get_command(&self, command_id: &str) -> Result<Option<(String, Vec<String>)>> {
        // Get a connection from the pool using the Repository trait method
        let mut conn = Repository::get_connection(self)?;

        // Query the database for the command
        let result = commands::table
            .filter(commands::id.eq(command_id))
            .select((commands::command, commands::args))
            .first::<(String, Vec<String>)>(&mut conn)
            .optional()
            .context("Failed to query command from database")?;

        Ok(result)
    }
}

// Implement Repository trait for CommandRepository by delegating to the base repository
impl Repository for CommandRepository {
    fn get_pool(&self) -> &DbPool {
        self.base.get_pool()
    }
}
