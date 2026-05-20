-- Explicit, scoped role grants. Each row links one Cedar policy-template
-- instance at control-plane startup: ?principal = user, ?resource = scope.
--
-- role_name must match a template @id in cedar/templates.cedar
-- (e.g. "project-admin", "organization-admin"). scope_kind constrains scope_id
-- to the matching tenancy entity.

CREATE TABLE permission_grants (
    id         TEXT        PRIMARY KEY,
    user_id    TEXT        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    role_name  TEXT        NOT NULL,
    scope_kind TEXT        NOT NULL CHECK (scope_kind IN ('organization', 'project')),
    scope_id   TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, role_name, scope_kind, scope_id)
);

CREATE INDEX permission_grants_user_id_idx ON permission_grants (user_id);
