-- Roles become first-class data: a named, editable bundle of permissions bound
-- to a scope kind. The Cedar policy set is generated from these rows, so editing
-- a role's permissions changes authorization on the next reload. Builtin roles
-- are seeded here with a stable id (== key); tenant-owned custom roles
-- (owner_org_id set) come later. This replaces the two compiled-in Cedar role
-- templates (full-control / agent) with data-driven per-role generation.

CREATE TABLE roles (
    id            TEXT        PRIMARY KEY,
    -- Stable identifier for builtin roles ("organization-admin"); NULL for
    -- tenant custom roles. Lets code/migrations reference a builtin without
    -- depending on its mutable display name.
    key           TEXT        UNIQUE,
    name          TEXT        NOT NULL,
    description   TEXT        NOT NULL DEFAULT '',
    scope_kind    TEXT        NOT NULL CHECK (scope_kind IN ('system', 'organization', 'project')),
    -- Owning organization for a tenant custom role; NULL = global (builtin).
    owner_org_id  TEXT        REFERENCES organizations (id) ON DELETE CASCADE,
    builtin       BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX roles_owner_org_idx ON roles (owner_org_id);

-- A role's permissions. Each row is a `Permission::key()` (e.g. 'runPipeline'),
-- or the sentinel '*' meaning full control (any action within the grant's
-- scope) — the latter maps to the unconstrained-action Cedar body.
CREATE TABLE role_permissions (
    role_id     TEXT NOT NULL REFERENCES roles (id) ON DELETE CASCADE,
    permission  TEXT NOT NULL,
    PRIMARY KEY (role_id, permission)
);

-- Seed the five builtin roles (id == key, stable). Admin roles confer full
-- control ('*'); agent roles confer only the job-execution permissions. This
-- mirrors the previous role/agent Cedar templates exactly, so authorization is
-- unchanged on a fresh DB.
INSERT INTO roles (id, key, name, description, scope_kind, builtin) VALUES
    ('system-admin',       'system-admin',       'System Admin',       'Global super-user: full control over every scope.',              'system',       TRUE),
    ('organization-admin', 'organization-admin', 'Organization Admin', 'Owner of an organization and everything beneath it.',            'organization', TRUE),
    ('project-admin',      'project-admin',      'Project Admin',      'Owner of a project and everything beneath it.',                  'project',      TRUE),
    ('organization-agent', 'organization-agent', 'Organization Agent', 'Machine app scoped to an organization: pull and run its jobs.', 'organization', TRUE),
    ('project-agent',      'project-agent',      'Project Agent',      'Machine app scoped to a project: pull and run its jobs.',        'project',      TRUE);

INSERT INTO role_permissions (role_id, permission) VALUES
    ('system-admin',       '*'),
    ('organization-admin', '*'),
    ('project-admin',      '*'),
    ('organization-agent', 'readPipeline'),
    ('organization-agent', 'executeJob'),
    ('organization-agent', 'writeJobStatus'),
    ('organization-agent', 'writeJobLog'),
    ('project-agent',      'readPipeline'),
    ('project-agent',      'executeJob'),
    ('project-agent',      'writeJobStatus'),
    ('project-agent',      'writeJobLog');
