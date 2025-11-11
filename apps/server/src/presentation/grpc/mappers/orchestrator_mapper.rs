use crate::application::dto::RunPipelineResponseDto;
use protocol::services::orchestrator::RunPipelineResponse;

impl From<RunPipelineResponseDto> for RunPipelineResponse {
    fn from(dto: RunPipelineResponseDto) -> Self {
        RunPipelineResponse {
            job_id: dto.job_id.to_string(),
        }
    }
}
