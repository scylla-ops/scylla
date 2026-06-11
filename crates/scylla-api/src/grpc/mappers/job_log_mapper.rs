use crate::grpc::convert::{ts, wrap};
use scylla_core::domain::entities::JobLog;
use scylla_core::domain::value_objects::job::LogStream;
use scylla_protocol::services::job::JobLogEntry;

pub fn job_log_to_proto(log: &JobLog) -> JobLogEntry {
    JobLogEntry {
        id: wrap(log.id().to_string()),
        job_id: wrap(log.job_id().to_string()),
        node_id: wrap(log.node_id().to_string()),
        stream: log_stream_str(log.stream()).to_string(),
        line: log.line().to_string(),
        timestamp: ts(log.timestamp()),
    }
}

fn log_stream_str(stream: &LogStream) -> &'static str {
    match stream {
        LogStream::Stdout => "stdout",
        LogStream::Stderr => "stderr",
    }
}
