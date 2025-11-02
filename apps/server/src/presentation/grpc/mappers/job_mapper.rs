use crate::application::dto::JobResponseDto;
use protocol::services::job::JobResponse;

impl From<JobResponseDto> for JobResponse {
    fn from(dto: JobResponseDto) -> Self {
        JobResponse {
            job_id: dto.id.to_string(),
            pipeline_id: dto.pipeline_id.to_string(),
            status: dto.status.to_string(),
            content: dto.content.to_string(),
            created_at: dto.created_at.to_rfc3339(),
            updated_at: dto.updated_at.to_rfc3339(),
        }
    }
}

impl From<&JobResponseDto> for JobResponse {
    fn from(dto: &JobResponseDto) -> Self {
        JobResponse {
            job_id: dto.id.to_string(),
            pipeline_id: dto.pipeline_id.to_string(),
            status: dto.status.to_string(),
            content: dto.content.to_string(),
            created_at: dto.created_at.to_rfc3339(),
            updated_at: dto.updated_at.to_rfc3339(),
        }
    }
}
