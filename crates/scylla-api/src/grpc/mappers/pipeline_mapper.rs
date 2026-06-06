use crate::grpc::convert::{ts, wrap};
use scylla_core::domain::entities::{Pipeline, PipelineNode as DomainPipelineNode};
use scylla_core::domain::value_objects::pipeline::{
    EnvSource, EnvVar as DomainEnvVar, NodeId, Shell, Step,
};
use scylla_protocol::services::common;
use scylla_protocol::services::pipeline::{
    EnvVar, PipelineNode, PipelineResponse, PipelineSummary, env_var, pipeline_node,
};

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
        working_dir: node
            .working_dir()
            .map(|wd| wd.as_str().to_string())
            .unwrap_or_default(),
        env: node.env().iter().map(env_to_proto).collect(),
        step: Some(step_to_proto(node.step())),
    }
}

fn env_to_proto(ev: &DomainEnvVar) -> EnvVar {
    let source = match ev.source() {
        EnvSource::Literal(v) => env_var::Source::Value(v.clone()),
        EnvSource::Secret(name) => env_var::Source::SecretRef(name.as_str().to_string()),
    };
    EnvVar {
        key: ev.key().to_string(),
        source: Some(source),
    }
}

fn step_to_proto(step: &Step) -> pipeline_node::Step {
    match step {
        Step::Exec { command, args } => pipeline_node::Step::Exec(common::ExecStep {
            command: command.clone(),
            args: args.clone(),
        }),
        Step::Script { script, shell } => pipeline_node::Step::Script(common::ScriptStep {
            script: script.clone(),
            shell: shell_to_proto(*shell) as i32,
        }),
    }
}

fn shell_to_proto(shell: Shell) -> common::Shell {
    match shell {
        Shell::Sh => common::Shell::Sh,
        Shell::Bash => common::Shell::Bash,
    }
}
