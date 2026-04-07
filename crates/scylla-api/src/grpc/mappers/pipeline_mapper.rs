use protocol::services::pipeline::{PipelineNode, PipelineResponse, PipelineSummary};
use scylla_core::domain::entities::{Pipeline, PipelineNode as DomainPipelineNode};
use scylla_core::domain::value_objects::pipeline::NodeId;

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

pub fn pipeline_to_proto_summary(pipeline: &Pipeline) -> PipelineSummary {
    PipelineSummary {
        pipeline_id: pipeline.id().to_string(),
        project_id: pipeline.project_id().to_string(),
        name: pipeline.name().to_string(),
        node_count: pipeline.nodes().len() as u32,
        created_at: pipeline.created_at().to_rfc3339(),
        updated_at: pipeline.updated_at().to_rfc3339(),
    }
}

pub fn pipeline_node_to_proto(node: &DomainPipelineNode) -> PipelineNode {

    PipelineNode {
        node_id: node.id().to_string(),
        deps: node.deps().iter().map(|d: &NodeId| d.to_string()).collect(),
        command: node.command().to_string(),
        args: node.args().to_vec(),
    }
}
