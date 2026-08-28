-- What an agent reports about its machine when it opens the stream (its
-- `AgentHello`). All nullable and written as one block: an agent that never
-- sends a hello — an older binary, or one that dies mid-handshake — simply
-- leaves them NULL, and presence keeps working off the stream as before.
--
-- `host_reported_at` doubles as the "has this agent ever introduced itself"
-- flag, so a partially-filled row is still readable.
ALTER TABLE agents
    ADD COLUMN agent_version    TEXT,
    ADD COLUMN host_os          TEXT,
    ADD COLUMN host_arch        TEXT,
    ADD COLUMN hostname         TEXT,
    ADD COLUMN cpu_count        INTEGER,
    ADD COLUMN total_memory_mb  BIGINT,
    ADD COLUMN host_reported_at TIMESTAMPTZ;
