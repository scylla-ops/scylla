use crate::api::v1::common::base::Repository;
use crate::api::v1::modules::teams::dto::{NewTeamRequest, TeamResponse};
use crate::api::v1::modules::teams::repository::TeamRepositoryTrait;
use anyhow::Result;
use uuid::Uuid;

pub struct TeamService<R: Repository + TeamRepositoryTrait> {
    repository: R,
}

pub trait TeamServiceTrait<R: Repository + TeamRepositoryTrait> {
    fn new(repository: R) -> Self;
    async fn create_team(&self, req: NewTeamRequest) -> Result<Uuid>;
    async fn get_team_by_id(&self, team_uuid: Uuid) -> Result<Option<TeamResponse>>;
    /*async fn get_all_teams(&self) -> Result<Vec<TeamResponse>>;
    async fn update_team_by_id(&self, team_uuid: Uuid, req: UpdateTeamRequest) -> Result<()>;
    async fn delete_team_by_id(&self, team_uuid: Uuid) -> Result<()>;*/
}

impl<R: Repository + TeamRepositoryTrait> TeamServiceTrait<R> for TeamService<R> {
    fn new(repository: R) -> Self {
        Self { repository }
    }

    async fn create_team(&self, req: NewTeamRequest) -> Result<Uuid> {
        self.repository.create_team(req.try_into()?).await
    }

    async fn get_team_by_id(&self, team_uuid: Uuid) -> Result<Option<TeamResponse>> {
        Ok(self
            .repository
            .get_team_by_uuid(team_uuid)
            .await?
            .map(TeamResponse::from))
    }
    /*
    async fn get_all_teams(&self) -> Result<Vec<TeamResponse>> {
        let teams = self.repository.get_all_teams().await?;
        Ok(teams.into_iter().map(TeamResponse::from).collect())
    }

    async fn update_team_by_id(&self, team_uuid: Uuid, req: UpdateTeamRequest) -> Result<()> {
        self.repository
            .update_team_by_uuid(team_uuid, req.try_into()?)
            .await?;
        Ok(())
    }

    async fn delete_team_by_id(&self, team_uuid: Uuid) -> Result<()> {
        self.repository.delete_team_by_uuid(team_uuid).await?;
        Ok(())
    }*/
}
