-- Agents represent connected workers. `last_seen_at` is bumped on every
-- heartbeat; `shutdown_at` is set when an agent leaves gracefully. The
-- domain's `Agent::is_connected` reads both. `hostname` is indexed
-- (non-unique) for the recorder's hostname-based lookups.

CREATE TABLE agents (
    id                       TEXT        PRIMARY KEY,
    hostname                 TEXT        NOT NULL,
    last_seen_at             TIMESTAMPTZ NOT NULL,
    shutdown_at              TIMESTAMPTZ NULL,
    heartbeat_interval_secs  BIGINT      NOT NULL,
    created_at               TIMESTAMPTZ NOT NULL,
    updated_at               TIMESTAMPTZ NOT NULL
);

CREATE INDEX agents_hostname_idx ON agents (hostname);
CREATE INDEX agents_last_seen_at_idx ON agents (last_seen_at DESC);
