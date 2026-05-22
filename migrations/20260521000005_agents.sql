-- A agent is a SPECIALIZED app: a machine principal (row in `apps`) that also
-- runs jobs. This table is the 1:1 extension marking "this app is a agent" and
-- holding agent-only attributes. A plain app has no row here. Presence
-- (online/offline) stays live in the in-memory agent registry; `last_seen` is
-- the durable "last activity" so it survives a control-plane restart.

CREATE TABLE agents (
    app_id     TEXT        PRIMARY KEY REFERENCES apps (id) ON DELETE CASCADE,
    last_seen  TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Which agent executed a job. Dispatch picks exactly one agent per job, so a
-- single nullable column suffices (NULL = pending / never dispatched). The FK
-- targets `agents` (not `apps`) to enforce "only a agent runs jobs". ON DELETE
-- SET NULL keeps the job history when a agent is removed — only the attribution
-- is dropped.
ALTER TABLE jobs
    ADD COLUMN agent_app_id TEXT NULL REFERENCES agents (app_id) ON DELETE SET NULL;

-- Agent stats are derived from jobs (count by status + last run); this index
-- covers "jobs of agent X, newest first".
CREATE INDEX jobs_agent_app_id_created_at_idx ON jobs (agent_app_id, created_at DESC);
