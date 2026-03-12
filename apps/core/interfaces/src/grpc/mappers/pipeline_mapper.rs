use domain::entities::Pipeline;
use protocol::services::pipeline::{PipelineNode, PipelineResponse};

pub fn pipeline_to_proto(pipeline: &Pipeline) -> PipelineResponse {
    PipelineResponse {
        pipeline_id: pipeline.id().to_string(),
        project_id: pipeline.project_id().to_string(),
        name: pipeline.name().to_string(),
        nodes: pipeline
            .nodes()
            .iter()
            .map(pipeline_node_to_proto)
            .collect(),
        created_at: pipeline.created_at().to_rfc3339(),
        updated_at: pipeline.updated_at().to_rfc3339(),
    }
}

pub fn pipeline_node_to_proto(node: &domain::entities::PipelineNode) -> PipelineNode {
    PipelineNode {
        node_id: node.id().to_string(),
        deps: node.deps().iter().map(|d| d.to_string()).collect(),
        command: node.command().to_string(),
        args: node.args().to_vec(),
    }
}
