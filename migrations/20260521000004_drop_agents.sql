-- The heartbeat-era `agents` registry is gone: an agent is now an `App` whose
-- liveness is the open worker stream (tracked in memory, not the database).
DROP TABLE IF EXISTS agents;
