use crate::grpc::convert::{ts, wrap};
use scylla_core::domain::entities::{Pipeline, PipelineNode as DomainPipelineNode};
use scylla_core::domain::value_objects::pipeline::NodeId;
use scylla_protocol::services::common;
use scylla_protocol::services::pipeline::{PipelineNode, PipelineResponse, PipelineSummary};

pub fn pipeline_to_proto(pipeline: &Pipeline) -> PipelineResponse {
    PipelineResponse {
        pipeline_id: wrap(pipeline.id().to_string()),
        project_id: wrap(pipeline.project_id().to_string()),
        name: pipeline.name().to_string(),
        nodes: pipeline
            .nodes()
            .iter()
            .map(pipeline_node_to_proto)
            .collect(),
        created_at: ts(pipeline.created_at()),
        updated_at: ts(pipeline.updated_at()),
    }
}

pub fn pipeline_to_proto_summary(pipeline: &Pipeline) -> PipelineSummary {
    PipelineSummary {
        pipeline_id: wrap(pipeline.id().to_string()),
        project_id: wrap(pipeline.project_id().to_string()),
        name: pipeline.name().to_string(),
        node_count: u32::try_from(pipeline.nodes().len()).unwrap_or(u32::MAX),
        created_at: ts(pipeline.created_at()),
        updated_at: ts(pipeline.updated_at()),
    }
}

pub fn pipeline_node_to_proto(node: &DomainPipelineNode) -> PipelineNode {
    PipelineNode {
        node_id: wrap(node.id().to_string()),
        deps: node
            .deps()
            .iter()
            .map(|d: &NodeId| common::NodeId {
                value: d.to_string(),
            })
            .collect(),
        command: node.command().to_string(),
        args: node.args().to_vec(),
    }
}
