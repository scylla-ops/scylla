use crate::grpc::convert::{ts, wrap};
use scylla_domain::domain::job::JobLog;
use scylla_proto::convert::log_stream_to_proto;
use scylla_proto::job::v1::JobLogEntry;

pub fn job_log_to_proto(log: &JobLog) -> JobLogEntry {
    JobLogEntry {
        job_log_id: wrap(log.id().to_string()),
        job_id: wrap(log.job_id().to_string()),
        node_id: wrap(log.node_id().to_string()),
        stream: log_stream_to_proto(*log.stream()) as i32,
        line: log.line().to_string(),
        timestamp: ts(log.timestamp()),
    }
}
