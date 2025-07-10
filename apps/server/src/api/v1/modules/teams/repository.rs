use crate::api::v1::common::base::{BaseRepository, Repository};
use crate::api::v1::models::teams::Team;
use crate::api::v1::modules::teams::dto::NewTeam;
use crate::database::DieselPool;
use anyhow::Context;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Repository)]
pub struct TeamRepository {
    base: BaseRepository,
}

pub trait TeamRepositoryTrait {
    async fn create_team(&self, new_team: NewTeam) -> anyhow::Result<Uuid>;
    async fn get_team_by_uuid(&self, team_uuid: Uuid) -> anyhow::Result<Option<Team>>;
    /*
    async fn get_all_teams(&self) -> anyhow::Result<Vec<Team>>;
    async fn update_team_by_uuid(
        &self,
        team_uuid: uuid::Uuid,
        updated_team: UpdateTeam,
    ) -> anyhow::Result<usize>;
    async fn delete_team_by_uuid(&self, team_uuid: uuid::Uuid) -> anyhow::Result<usize>;*/
}

impl TeamRepositoryTrait for TeamRepository {
    async fn create_team(&self, new_team: NewTeam) -> anyhow::Result<uuid::Uuid> {
        use crate::database::schema::teams::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        let uuid = diesel::insert_into(teams)
            .values(&new_team)
            .returning(id)
            .get_result(&mut conn)
            .context("Failed to insert new team")?;

        tracing::debug!("Inserted new team with uuid: {}", uuid);
        Ok(uuid)
    }

    async fn get_team_by_uuid(&self, team_uuid: Uuid) -> anyhow::Result<Option<Team>> {
        use crate::database::schema::teams::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        let team = teams
            .filter(id.eq(team_uuid))
            .first::<Team>(&mut conn)
            .optional()
            .context("Failed to fetch team by UUID")?;

        if let Some(ref t) = team {
            tracing::debug!("Found team: {:?}", t);
        } else {
            tracing::debug!("No team found with UUID: {}", team_uuid);
        }

        Ok(team)
    }

    // Other methods would be implemented similarly...
}
