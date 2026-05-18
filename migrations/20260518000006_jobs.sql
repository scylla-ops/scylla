-- Jobs are runtime executions of pipelines. `status` is TEXT (not an enum
-- type) to keep migrations cheap when the JobStatus enum gains a variant —
-- domain validation in `JobStatus::new` is the source of truth.
-- `node_executions` is JSONB for the same reason as `pipelines.nodes`: the
-- per-node state travels with the job, no cross-job query filters on it.
--
-- `job_logs` keeps individual log lines in a separate table because they're
-- append-heavy and queried by (job_id, timestamp). The composite index also
-- supports the (job_id, node_id, timestamp) access pattern from streaming.

CREATE TABLE jobs (
    id              TEXT        PRIMARY KEY,
    pipeline_id     TEXT        NOT NULL REFERENCES pipelines (id) ON DELETE CASCADE,
    status          TEXT        NOT NULL,
    node_executions JSONB       NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL,
    started_at      TIMESTAMPTZ NULL,
    finished_at     TIMESTAMPTZ NULL
);

CREATE INDEX jobs_pipeline_id_idx ON jobs (pipeline_id);
CREATE INDEX jobs_status_idx ON jobs (status);
CREATE INDEX jobs_created_at_idx ON jobs (created_at DESC);

CREATE TABLE job_logs (
    id              TEXT        PRIMARY KEY,
    job_id          TEXT        NOT NULL REFERENCES jobs (id) ON DELETE CASCADE,
    node_id         TEXT        NOT NULL,
    stream          TEXT        NOT NULL,
    line            TEXT        NOT NULL,
    timestamp       TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL
);

CREATE INDEX job_logs_job_id_timestamp_idx ON job_logs (job_id, timestamp);
CREATE INDEX job_logs_job_id_node_id_timestamp_idx ON job_logs (job_id, node_id, timestamp);
