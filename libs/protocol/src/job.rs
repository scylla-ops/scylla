use crate::pipeline::{PStage, PStep, Pipeline};
use crate::shell::Shell;
use crate::{Deserialize, Serialize};
use derive_builder::Builder;
use derive_more::Constructor;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutionStatus {
    #[default]
    Queued,
    Running,
    Succeeded,
    Failed,
    Canceled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEntry {
    pub id: String, // DB ID can be anything
    pub job: Job,   // Job definition
}

// Fully qualified (merge of a pipeline and db record)
#[derive(Debug, Clone, Serialize, Deserialize, Constructor, Builder)]
pub struct Job {
    pub state: ExecutionStatus,
    pub name: String,
    pub stages: Vec<JobStage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Constructor, Builder)]
pub struct JobStage {
    #[serde(default = "Uuid::new_v4")]
    pub uuid: Uuid,
    pub state: ExecutionStatus,
    pub name: String,
    pub steps: Vec<JobStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Constructor, Builder)]
pub struct JobStep {
    #[serde(default = "Uuid::new_v4")]
    pub uuid: Uuid,
    pub state: ExecutionStatus,
    pub name: String,
    pub shell: Shell,
    pub command: String,
    pub args: Vec<String>,
}

// DB
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobData {
    pub state: ExecutionStatus,
    pub stages: Vec<StageData>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StageData {
    pub uuid: Uuid,
    pub state: ExecutionStatus,
    pub steps: Vec<StepData>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StepData {
    pub uuid: Uuid,
    pub state: ExecutionStatus,
}

impl From<Pipeline> for JobData {
    fn from(value: Pipeline) -> Self {
        JobData {
            state: Default::default(),
            stages: value
                .stages
                .into_iter()
                .map(|stage| StageData {
                    uuid: Uuid::new_v4(),
                    state: Default::default(),
                    steps: stage
                        .steps
                        .into_iter()
                        .map(|_| StepData {
                            uuid: Uuid::new_v4(),
                            state: Default::default(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

impl From<Job> for JobData {
    fn from(value: Job) -> Self {
        JobData {
            state: value.state,
            stages: value
                .stages
                .into_iter()
                .map(|stage| StageData {
                    uuid: stage.uuid,
                    state: stage.state,
                    steps: stage
                        .steps
                        .into_iter()
                        .map(|step| StepData {
                            uuid: step.uuid,
                            state: step.state,
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

impl From<Job> for Pipeline {
    fn from(value: Job) -> Self {
        Pipeline {
            name: value.name,
            stages: value
                .stages
                .into_iter()
                .map(|stage| PStage {
                    name: stage.name,
                    steps: stage
                        .steps
                        .into_iter()
                        .map(|step| PStep {
                            name: step.name,
                            shell: step.shell,
                            command: step.command,
                            args: step.args,
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

impl Job {
    pub fn from_pipeline_and_data(p: Pipeline, job_rec: JobData) -> Job {
        let mut jc_stages_iter = job_rec.stages.into_iter();

        let mut job_stages = Vec::with_capacity(p.stages.len());
        for p_stage in p.stages {
            let jc_stage_opt = jc_stages_iter.next();

            let (stage_uuid, stage_state, jc_steps_iter) = if let Some(js) = jc_stage_opt {
                (js.uuid, js.state, Some(js.steps.into_iter()))
            } else {
                (Uuid::new_v4(), ExecutionStatus::default(), None)
            };

            let mut steps_vec = Vec::with_capacity(p_stage.steps.len());
            let mut jc_steps_iter = jc_steps_iter.unwrap_or_else(|| Vec::new().into_iter());

            for p_step in p_stage.steps {
                let (step_uuid, step_state) = if let Some(js) = jc_steps_iter.next() {
                    (js.uuid, js.state)
                } else {
                    (Uuid::new_v4(), ExecutionStatus::default())
                };

                steps_vec.push(JobStep {
                    uuid: step_uuid,
                    state: step_state,
                    name: p_step.name,
                    shell: p_step.shell,
                    command: p_step.command,
                    args: p_step.args,
                });
            }

            job_stages.push(JobStage {
                uuid: stage_uuid,
                state: stage_state,
                name: p_stage.name,
                steps: steps_vec,
            });
        }

        Job {
            state: job_rec.state,
            name: p.name,
            stages: job_stages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_pipeline_and_job_record() {
        let pipeline = Pipeline {
            name: "Test Pipeline".to_string(),
            stages: vec![PStage {
                name: "Stage 1".to_string(),
                steps: vec![PStep {
                    name: "Step 1".to_string(),
                    shell: Shell::Bash,
                    command: "echo".to_string(),
                    args: vec!["hello".to_string()],
                }],
            }],
        };

        let stage_uuid = Uuid::new_v4();
        let step_uuid = Uuid::new_v4();

        let job_record = JobData {
            state: ExecutionStatus::Running,
            stages: vec![StageData {
                uuid: stage_uuid,
                state: ExecutionStatus::Running,
                steps: vec![StepData {
                    uuid: step_uuid,
                    state: ExecutionStatus::Running,
                }],
            }],
        };

        let job = Job::from_pipeline_and_data(pipeline, job_record);

        assert_eq!(job.name, "Test Pipeline");
        assert_eq!(job.state, ExecutionStatus::Running);
        assert_eq!(job.stages.len(), 1);
        assert_eq!(job.stages[0].name, "Stage 1");
        assert_eq!(job.stages[0].state, ExecutionStatus::Running);
        assert_eq!(job.stages[0].steps.len(), 1);
        assert_eq!(job.stages[0].steps[0].name, "Step 1");
        assert_eq!(job.stages[0].steps[0].state, ExecutionStatus::Running);
        assert_eq!(job.stages[0].steps[0].shell, Shell::Bash);
        assert_eq!(job.stages[0].steps[0].command, "echo");
        assert_eq!(job.stages[0].steps[0].args, vec!["hello".to_string()]);
        assert_eq!(job.stages[0].uuid, stage_uuid);
        assert_eq!(job.stages[0].steps[0].uuid, step_uuid);
    }
}
