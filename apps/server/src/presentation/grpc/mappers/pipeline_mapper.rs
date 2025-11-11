use crate::application::dto::PipelineResponseDto;
use protocol::services::pipeline::PipelineResponse;

impl From<PipelineResponseDto> for PipelineResponse {
    fn from(dto: PipelineResponseDto) -> Self {
        PipelineResponse {
            pipeline_id: dto.id.to_string(),
            content: dto.content.to_string(),
            created_at: dto.created_at.to_rfc3339(),
            updated_at: dto.updated_at.to_rfc3339(),
        }
    }
}
