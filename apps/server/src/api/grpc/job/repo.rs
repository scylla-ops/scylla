use crate::api::base::BaseRepository;
use crate::api::base::diesel_repo_base::Repository;
use crate::api::grpc::job::JobRepository;
use crate::database::DieselPool;
use anyhow::Context;
use diesel::RunQueryDsl;
use protocol::pipeline::Pipeline;
use repository_derive::DieselRepository;
use uuid::Uuid;

#[derive(DieselRepository)]
pub struct JobRepositoryDiesel {
    base: BaseRepository,
}

#[async_trait::async_trait]
impl JobRepository for JobRepositoryDiesel {
    async fn create_job(&self, snapshot_id: Uuid, pipeline: Pipeline) -> anyhow::Result<Uuid> {
        use crate::api::grpc::job::models::NewJob;
        use crate::api::grpc::job::models::NewStage;
        use crate::api::grpc::job::models::NewStep;
        use crate::database::schema::jobs;
        use crate::database::schema::stages;
        use crate::database::schema::steps;
        use diesel::Connection;

        let mut conn = Repository::get_connection(self)?;

        let job_id = conn.transaction(|conn| {
            let new_job = NewJob {
                pipeline_snapshot_id: snapshot_id,
            };

            let job_id = diesel::insert_into(jobs::table)
                .values(&new_job)
                .returning(jobs::dsl::id)
                .get_result::<Uuid>(conn)
                .context("Failed to create job")?;

            for (stage_idx, pstage) in pipeline.stages.iter().enumerate() {
                let new_stage = NewStage {
                    job_id,
                    position: stage_idx as i32,
                };

                let stage_id = diesel::insert_into(stages::table)
                    .values(&new_stage)
                    .returning(stages::dsl::id)
                    .get_result::<Uuid>(conn)
                    .context("Failed to create stage")?;

                for (step_idx, _) in pstage.steps.iter().enumerate() {
                    let new_step = NewStep {
                        stage_id,
                        position: step_idx as i32,
                    };

                    diesel::insert_into(steps::table)
                        .values(&new_step)
                        .execute(conn)
                        .context("Failed to create step")?;
                }
            }

            Ok::<Uuid, anyhow::Error>(job_id)
        })?;

        Ok(job_id)
    }
}