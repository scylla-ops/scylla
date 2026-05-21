#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("invalid control-plane URL `{url}`: {message}")]
    InvalidUrl { url: String, message: String },

    #[error("failed to connect to control plane: {0}")]
    Connection(#[from] tonic::transport::Error),

    #[error("worker stream closed unexpectedly")]
    StreamClosed,

    #[error("gRPC error: {0}")]
    Status(#[from] tonic::Status),

    #[error("invalid bearer token metadata: {0}")]
    InvalidToken(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("node {node_id} failed with exit code {exit_code}")]
    NodeFailed { node_id: String, exit_code: i32 },

    #[error("node {node_id} was killed by signal")]
    NodeKilled { node_id: String },

    #[error("node {node_id} cancelled by executor")]
    Cancelled { node_id: String },

    #[error("dangling dependencies — not all nodes could be scheduled (possible cycle)")]
    DanglingDeps,

    #[error("failed to spawn command: {0}")]
    Spawn(#[source] std::io::Error),

    #[error("failed to publish status: {0}")]
    Publish(String),

    #[error("node task panicked: {message}")]
    NodeTaskPanic { message: String },
}
