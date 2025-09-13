use crate::api::base::BaseRepository;
use crate::api::base::diesel_repo_base::Repository;
use crate::api::grpc::job::{JobRepository, StageRepository, StepRepository};
use crate::database::DieselPool;
use repository_derive::DieselRepository;
use uuid::Uuid;

#[derive(DieselRepository)]
pub struct JobRepositoryDiesel {
    base: BaseRepository,
}

#[derive(DieselRepository)]
pub struct StageRepositoryDiesel {
    base: BaseRepository,
}

#[derive(DieselRepository)]
pub struct StepRepositoryDiesel {
    base: BaseRepository,
}

#[async_trait::async_trait]
impl JobRepository for JobRepositoryDiesel {
    async fn create_job(&self, pipeline_id: Uuid) -> anyhow::Result<Uuid> {
        todo!()
    }
}

#[async_trait::async_trait]
impl StageRepository for StageRepositoryDiesel {
    async fn create_stages(&self, job_id: Uuid) -> anyhow::Result<()> {
        todo!()
    }
}

#[async_trait::async_trait]
impl StepRepository for StepRepositoryDiesel {
    async fn create_steps(&self, stage_id: Uuid) -> anyhow::Result<()> {
        todo!()
    }
}
