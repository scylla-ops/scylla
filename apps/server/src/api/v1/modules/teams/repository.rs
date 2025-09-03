use crate::api::v1::common::base::{BaseRepository, DieselRepository};
use crate::api::v1::models::teams::Team;
use crate::api::v1::modules::teams::dto::{NewTeam, UpdateTeam};
use crate::database::DieselPool;
use anyhow::Context;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(DieselRepository)]
pub struct TeamRepository {
    base: BaseRepository,
}

pub trait TeamRepositoryTrait {
    async fn create_team(&self, new_team: NewTeam) -> anyhow::Result<Uuid>;
    async fn get_team_by_uuid(&self, team_uuid: Uuid) -> anyhow::Result<Option<Team>>;
    async fn get_all_teams(&self) -> anyhow::Result<Vec<Team>>;
    async fn update_team_by_uuid(
        &self,
        team_uuid: Uuid,
        updated_team: UpdateTeam,
    ) -> anyhow::Result<usize>;
    async fn delete_team_by_uuid(&self, team_uuid: Uuid) -> anyhow::Result<usize>;
}

impl TeamRepositoryTrait for TeamRepository {
    async fn create_team(&self, new_team: NewTeam) -> anyhow::Result<Uuid> {
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

    async fn get_all_teams(&self) -> anyhow::Result<Vec<Team>> {
        use crate::database::schema::teams::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        let teams_list = teams
            .load::<Team>(&mut conn)
            .context("Failed to fetch all teams")?;

        tracing::debug!("Fetched {} teams", teams_list.len());
        Ok(teams_list)
    }

    async fn update_team_by_uuid(
        &self,
        team_uuid: Uuid,
        updated_team: UpdateTeam,
    ) -> anyhow::Result<usize> {
        use crate::database::schema::teams::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        let updated_count = diesel::update(teams.filter(id.eq(team_uuid)))
            .set(&updated_team)
            .execute(&mut conn)
            .context("Failed to update team by UUID")?;

        if updated_count > 0 {
            tracing::debug!("Updated {} team(s) with UUID: {}", updated_count, team_uuid);
        } else {
            tracing::debug!("No team found with UUID: {}", team_uuid);
        }

        Ok(updated_count)
    }

    async fn delete_team_by_uuid(&self, team_uuid: Uuid) -> anyhow::Result<usize> {
        use crate::database::schema::teams::dsl::*;

        let mut conn = Repository::get_connection(self)?;

        let deleted_count = diesel::delete(teams.filter(id.eq(team_uuid)))
            .execute(&mut conn)
            .context("Failed to delete team by UUID")?;

        if deleted_count > 0 {
            tracing::debug!("Deleted {} team(s) with UUID: {}", deleted_count, team_uuid);
        } else {
            tracing::debug!("No team found with UUID: {}", team_uuid);
        }

        Ok(deleted_count)
    }
}
