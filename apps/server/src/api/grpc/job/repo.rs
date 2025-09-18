use crate::api::base::BaseRepository;
use crate::api::base::diesel_repo_base::Repository;
use crate::api::grpc::job::JobRepository;
use crate::api::grpc::job::models::{JobStatusUpdate, StageStatusUpdate, StepStatusUpdate};
use crate::database::DieselPool;
use anyhow::Context;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use protocol::job::Job;
use repository_derive::DieselRepository;
use uuid::Uuid;

#[derive(DieselRepository)]
pub struct JobRepositoryDiesel {
    base: BaseRepository,
}

#[async_trait::async_trait]
impl JobRepository for JobRepositoryDiesel {
    async fn create_job(&self, snapshot_id: Uuid, job: &Job) -> anyhow::Result<Uuid> {
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
                id: job.id,
                pipeline_snapshot_id: snapshot_id,
            };

            let job_id = diesel::insert_into(jobs::table)
                .values(&new_job)
                .returning(jobs::dsl::id)
                .get_result::<Uuid>(conn)
                .context("Failed to create job")?;

            for (stage_idx, pstage) in job.stages.iter().enumerate() {
                let new_stage = NewStage {
                    id: pstage.id,
                    job_id,
                    position: stage_idx as i32,
                };

                let stage_id = diesel::insert_into(stages::table)
                    .values(&new_stage)
                    .returning(stages::dsl::id)
                    .get_result::<Uuid>(conn)
                    .context("Failed to create stage")?;

                for (step_idx, step) in pstage.steps.iter().enumerate() {
                    let new_step = NewStep {
                        id: step.id,
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

    async fn update_job(&self, job_id: Uuid, updated_job: JobStatusUpdate) -> anyhow::Result<()> {
        use crate::database::schema::jobs;
        use diesel::RunQueryDsl;
        let mut conn = Repository::get_connection(self)?;

        diesel::update(jobs::table)
            .filter(jobs::id.eq(job_id))
            .set(updated_job)
            .execute(&mut conn)
            .context("Failed to update job")?;

        Ok(())
    }

    async fn update_stage(
        &self,
        stage_id: Uuid,
        updated_stage: StageStatusUpdate,
    ) -> anyhow::Result<()> {
        use crate::database::schema::stages;
        use diesel::RunQueryDsl;
        let mut conn = Repository::get_connection(self)?;

        diesel::update(stages::table)
            .filter(stages::id.eq(stage_id))
            .set(updated_stage)
            .execute(&mut conn)
            .context("Failed to update stage")?;

        Ok(())
    }

    async fn update_step(
        &self,
        step_id: Uuid,
        updated_step: StepStatusUpdate,
    ) -> anyhow::Result<()> {
        use crate::database::schema::steps;
        use diesel::RunQueryDsl;
        let mut conn = Repository::get_connection(self)?;

        diesel::update(steps::table)
            .filter(steps::id.eq(step_id))
            .set(updated_step)
            .execute(&mut conn)
            .context("Failed to update step")?;

        Ok(())
    }
}
