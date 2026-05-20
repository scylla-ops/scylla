-- Email-based invitations to join an organization. The table exists in every
-- edition; only the code that uses it is gated behind the `invitations` cargo
-- feature. `role_name` is optional: when set, accepting the invite also mints a
-- scoped grant (e.g. organization-admin).

CREATE TABLE organization_invites (
    id              TEXT        PRIMARY KEY,
    organization_id TEXT        NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    email           TEXT        NOT NULL,
    role_name       TEXT        NULL,
    token           TEXT        NOT NULL UNIQUE,
    status          TEXT        NOT NULL DEFAULT 'pending',
    invited_by      TEXT        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    expires_at      TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL
);

CREATE INDEX organization_invites_org_idx ON organization_invites (organization_id);
CREATE INDEX organization_invites_token_idx ON organization_invites (token);
