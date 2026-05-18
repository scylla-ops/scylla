-- Organizations are the top-level tenant in the domain. The N:M relation with
-- users is a plain junction table (no surrogate id) — composite PK is the
-- natural key, no need for a separate ULID.

CREATE TABLE organizations (
    id              TEXT        PRIMARY KEY,
    name            TEXT        NOT NULL,
    description     TEXT        NULL,
    is_active       BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL
);

CREATE INDEX organizations_created_at_idx ON organizations (created_at DESC);
CREATE INDEX organizations_name_idx ON organizations (name);

CREATE TABLE user_organization (
    user_id         TEXT        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    organization_id TEXT        NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, organization_id)
);

CREATE INDEX user_organization_organization_id_idx ON user_organization (organization_id);
