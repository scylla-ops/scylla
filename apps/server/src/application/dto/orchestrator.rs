use crate::domain::value_objects::{JobId, PipelineId};

#[derive(Debug, Clone)]
pub struct RunPipelineRequestDto {
    pub pipeline_id: PipelineId,
}

#[derive(Debug, Clone)]
pub struct RunPipelineResponseDto {
    pub job_id: JobId,
}
