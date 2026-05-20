-- Runtime-managed Cedar policies. Merged with the static, compiled-in policy
-- set at boot and on every hot-reload. Each row is a permit rule validated
-- against the schema before it is persisted (validate-on-write); the live
-- PolicySet is rebuilt atomically when a row changes.
CREATE TABLE cedar_policies (
    id          TEXT        PRIMARY KEY,
    description TEXT        NOT NULL,
    text        TEXT        NOT NULL,
    enabled     BOOLEAN     NOT NULL DEFAULT TRUE,
    created_by  TEXT        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX cedar_policies_enabled_idx ON cedar_policies (enabled);
