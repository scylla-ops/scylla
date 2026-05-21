-- Machine principals (agents / automations) owned by an organization. The
-- credential is stored only as a hash; the plaintext secret is shown once at
-- creation. An app's authorization comes from permission_grants (typically the
-- worker role on its organization), not from this table.

CREATE TABLE apps (
    id              TEXT        PRIMARY KEY,
    organization_id TEXT        NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    secret_hash     TEXT        NOT NULL,
    is_active       BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (organization_id, name)
);

CREATE INDEX apps_organization_id_idx ON apps (organization_id);
