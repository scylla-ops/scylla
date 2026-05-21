-- A worker is a SPECIALIZED app: a machine principal (row in `apps`) that also
-- runs jobs. This table is the 1:1 extension marking "this app is a worker" and
-- holding worker-only attributes. A plain app has no row here. Presence
-- (online/offline) stays live in the in-memory worker registry; `last_seen` is
-- the durable "last activity" so it survives a control-plane restart.

CREATE TABLE workers (
    app_id     TEXT        PRIMARY KEY REFERENCES apps (id) ON DELETE CASCADE,
    last_seen  TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Which worker executed a job. Dispatch picks exactly one worker per job, so a
-- single nullable column suffices (NULL = pending / never dispatched). The FK
-- targets `workers` (not `apps`) to enforce "only a worker runs jobs". ON DELETE
-- SET NULL keeps the job history when a worker is removed — only the attribution
-- is dropped.
ALTER TABLE jobs
    ADD COLUMN worker_app_id TEXT NULL REFERENCES workers (app_id) ON DELETE SET NULL;

-- Worker stats are derived from jobs (count by status + last run); this index
-- covers "jobs of worker X, newest first".
CREATE INDEX jobs_worker_app_id_created_at_idx ON jobs (worker_app_id, created_at DESC);
