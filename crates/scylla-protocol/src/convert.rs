//! Conversions between the kernel types and their wire form.
//!
//! These live here, and not on either side, because they are the one piece of
//! code the control plane and the agent must agree on exactly. Each of them used
//! to exist twice, once per binary: the agent encoded a [`JobEvent`] into a
//! `JobStatus` while the control plane decoded the same message back, and
//! nothing tied the two halves together. Adding a variant meant remembering to
//! edit a file in the other crate, and no test would have noticed the omission.
//!
//! `scylla-protocol` is the only crate both binaries link, which makes it the
//! natural home. It depends on `scylla-core` for the kernel types; the kernel
//! depends on nothing here, so there is no cycle.

use chrono::{DateTime, Utc};
use prost_types::Timestamp;

use scylla_core::JobEvent;
use scylla_core::domain::job::LogStream;
use scylla_core::domain::pipeline::Shell;

use crate::agent::v1::JobStatus;
use crate::agent::v1::job_status::{
    Event, JobCompleted, JobFailed, JobStarted, NodeCompleted, NodeFailed, NodeSkipped, NodeStarted,
};
use crate::common::v1 as common;
use crate::exec::v1 as exec;

/// A `DateTime<Utc>` as a protobuf `Timestamp`.
#[must_use]
pub fn timestamp(dt: DateTime<Utc>) -> Option<Timestamp> {
    Some(Timestamp {
        seconds: dt.timestamp(),
        nanos: i32::try_from(dt.timestamp_subsec_nanos()).unwrap_or(0),
    })
}

/// Encode a lifecycle event as the `JobStatus` an agent reports.
///
/// Each arm builds exactly the oneof variant carrying this event's fields: a
/// job-level event names no node, a node event always names one.
#[must_use]
pub fn job_event_to_status(job_id: &str, event: JobEvent) -> JobStatus {
    let node = |value: String| Some(common::NodeId { value });
    let event = match event {
        JobEvent::JobStarted => Event::JobStarted(JobStarted {}),
        JobEvent::NodeStarted { node_id } => Event::NodeStarted(NodeStarted {
            node_id: node(node_id),
        }),
        JobEvent::NodeCompleted { node_id } => Event::NodeCompleted(NodeCompleted {
            node_id: node(node_id),
        }),
        JobEvent::NodeFailed { node_id, error } => Event::NodeFailed(NodeFailed {
            node_id: node(node_id),
            error,
        }),
        JobEvent::NodeSkipped { node_id } => Event::NodeSkipped(NodeSkipped {
            node_id: node(node_id),
        }),
        JobEvent::JobCompleted => Event::JobCompleted(JobCompleted {}),
        JobEvent::JobFailed { error } => Event::JobFailed(JobFailed { error }),
    };
    JobStatus {
        job_id: Some(common::JobId {
            value: job_id.to_string(),
        }),
        event: Some(event),
    }
}

/// Decode a reported `JobStatus` back into a lifecycle event.
///
/// Returns `None` when the oneof is absent, which is a malformed report rather
/// than a valid state; the caller logs the skip.
#[must_use]
pub fn status_to_job_event(status: &JobStatus) -> Option<JobEvent> {
    let node_id = |id: &Option<common::NodeId>| id.clone().unwrap_or_default().value;
    Some(match status.event.as_ref()? {
        Event::JobStarted(_) => JobEvent::JobStarted,
        Event::NodeStarted(e) => JobEvent::NodeStarted {
            node_id: node_id(&e.node_id),
        },
        Event::NodeCompleted(e) => JobEvent::NodeCompleted {
            node_id: node_id(&e.node_id),
        },
        Event::NodeFailed(e) => JobEvent::NodeFailed {
            node_id: node_id(&e.node_id),
            error: e.error.clone(),
        },
        Event::NodeSkipped(e) => JobEvent::NodeSkipped {
            node_id: node_id(&e.node_id),
        },
        Event::JobCompleted(_) => JobEvent::JobCompleted,
        Event::JobFailed(e) => JobEvent::JobFailed {
            error: e.error.clone(),
        },
    })
}

/// A log stream as its proto enum.
#[must_use]
pub const fn log_stream_to_proto(stream: LogStream) -> common::LogStream {
    match stream {
        LogStream::Stdout => common::LogStream::Stdout,
        LogStream::Stderr => common::LogStream::Stderr,
    }
}

/// A proto log stream as the kernel type. `UNSPECIFIED` and anything unknown
/// fall back to stdout, matching how an unlabelled line has always been treated.
#[must_use]
pub fn log_stream_from_proto(raw: i32) -> LogStream {
    match common::LogStream::try_from(raw) {
        Ok(common::LogStream::Stderr) => LogStream::Stderr,
        _ => LogStream::Stdout,
    }
}

/// A shell as its proto enum.
#[must_use]
pub const fn shell_to_proto(shell: Shell) -> exec::Shell {
    match shell {
        Shell::Sh => exec::Shell::Sh,
        Shell::Bash => exec::Shell::Bash,
    }
}

/// A proto shell as the kernel type. `UNSPECIFIED` and anything unknown fall
/// back to `sh`, the shell guaranteed present in the agent image.
#[must_use]
pub fn shell_from_proto(raw: i32) -> Shell {
    match exec::Shell::try_from(raw) {
        Ok(exec::Shell::Bash) => Shell::Bash,
        _ => Shell::Sh,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn every_variant() -> Vec<JobEvent> {
        let node = || "build".to_string();
        vec![
            JobEvent::JobStarted,
            JobEvent::NodeStarted { node_id: node() },
            JobEvent::NodeCompleted { node_id: node() },
            JobEvent::NodeFailed {
                node_id: node(),
                error: "exit 1".to_string(),
            },
            JobEvent::NodeSkipped { node_id: node() },
            JobEvent::JobCompleted,
            JobEvent::JobFailed {
                error: "cancelled".to_string(),
            },
        ]
    }

    /// The encode and decode halves used to live in two different crates with
    /// nothing tying them together. This is the test that was impossible then.
    #[test]
    fn every_job_event_survives_a_round_trip() {
        for event in every_variant() {
            let wire = job_event_to_status("job-1", event.clone());
            let back = status_to_job_event(&wire).expect("a built status always carries its event");
            assert_eq!(
                format!("{event:?}"),
                format!("{back:?}"),
                "{event:?} did not survive the round trip"
            );
        }
    }

    /// Guards against a variant being added to JobEvent and handled on only one
    /// side. Both matches are exhaustive, so a new variant fails the build here.
    #[test]
    fn every_variant_is_covered() {
        assert_eq!(
            every_variant().len(),
            7,
            "add the new JobEvent variant here"
        );
    }

    #[test]
    fn a_status_without_an_event_is_rejected() {
        let empty = JobStatus {
            job_id: None,
            event: None,
        };
        assert!(status_to_job_event(&empty).is_none());
    }

    #[test]
    fn log_streams_round_trip() {
        for stream in [LogStream::Stdout, LogStream::Stderr] {
            let raw = log_stream_to_proto(stream) as i32;
            assert_eq!(log_stream_from_proto(raw), stream);
        }
        // Unknown and unspecified both mean stdout.
        assert_eq!(log_stream_from_proto(0), LogStream::Stdout);
        assert_eq!(log_stream_from_proto(99), LogStream::Stdout);
    }

    #[test]
    fn shells_round_trip() {
        for shell in [Shell::Sh, Shell::Bash] {
            let raw = shell_to_proto(shell) as i32;
            assert_eq!(shell_from_proto(raw), shell);
        }
        assert_eq!(shell_from_proto(0), Shell::Sh);
        assert_eq!(shell_from_proto(99), Shell::Sh);
    }
}
