use crate::grpc::convert::{log_stream_to_proto, ts, wrap};
use scylla_core::domain::job::JobLog;
use scylla_core::domain::job::LogStream;
use scylla_protocol::job::v1::JobLogEntry;

pub fn job_log_to_proto(log: &JobLog) -> JobLogEntry {
    JobLogEntry {
        job_log_id: wrap(log.id().to_string()),
        job_id: wrap(log.job_id().to_string()),
        node_id: wrap(log.node_id().to_string()),
        stream: log_stream_to_proto(log_stream_str(log.stream())) as i32,
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
