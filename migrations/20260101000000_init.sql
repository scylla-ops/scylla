-- Scylla initial schema (consolidated). The incremental migration history was
-- squashed into this single baseline — there is no deployed instance to
-- preserve it for. Tables are ordered so inline foreign keys resolve at creation
-- time. Keep this the single source of truth for the schema.

-- ── Users & auth ────────────────────────────────────────────────────────────

CREATE TABLE users (
    id            TEXT        PRIMARY KEY,
    username      TEXT        NOT NULL,
    password_hash TEXT        NOT NULL,
    is_active     BOOLEAN     NOT NULL DEFAULT TRUE,
    email         TEXT,
    created_at    TIMESTAMPTZ NOT NULL,
    updated_at    TIMESTAMPTZ NOT NULL
);
CREATE UNIQUE INDEX users_username_key ON users (username);
CREATE UNIQUE INDEX users_email_key ON users (email) WHERE email IS NOT NULL;
CREATE INDEX users_created_at_idx ON users (created_at DESC);

CREATE TABLE sessions (
    id             TEXT        PRIMARY KEY,
    token          TEXT        NOT NULL,
    user_id        TEXT        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at     TIMESTAMPTZ NOT NULL,
    expires_at     TIMESTAMPTZ NOT NULL,
    last_active_at TIMESTAMPTZ NOT NULL
);
CREATE UNIQUE INDEX sessions_token_key ON sessions (token);
CREATE INDEX sessions_user_id_idx ON sessions (user_id);
CREATE INDEX sessions_expires_at_idx ON sessions (expires_at);

CREATE TABLE user_oauth_identities (
    user_id          TEXT        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    provider         TEXT        NOT NULL,
    provider_user_id TEXT        NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (provider, provider_user_id)
);
CREATE INDEX user_oauth_identities_user_idx ON user_oauth_identities (user_id);

-- ── Tenancy: organizations → projects ─────────────────────────────────────────

CREATE TABLE organizations (
    id          TEXT        PRIMARY KEY,
    name        TEXT        NOT NULL,
    description TEXT,
    is_active   BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL
);
CREATE INDEX organizations_created_at_idx ON organizations (created_at DESC);
CREATE INDEX organizations_name_idx ON organizations (name);

CREATE TABLE user_organization (
    user_id         TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, organization_id)
);
CREATE INDEX user_organization_organization_id_idx ON user_organization (organization_id);

CREATE TABLE projects (
    id              TEXT        PRIMARY KEY,
    name            TEXT        NOT NULL,
    description     TEXT,
    organization_id TEXT        NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    is_active       BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL
);
CREATE INDEX projects_created_at_idx ON projects (created_at DESC);
CREATE INDEX projects_organization_id_idx ON projects (organization_id);

CREATE TABLE user_project (
    user_id    TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    project_id TEXT NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, project_id)
);
CREATE INDEX user_project_project_id_idx ON user_project (project_id);

-- ── Pipelines ────────────────────────────────────────────────────────────────

CREATE TABLE pipelines (
    id         TEXT        PRIMARY KEY,
    project_id TEXT        NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    name       TEXT        NOT NULL,
    nodes      JSONB       NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX pipelines_created_at_idx ON pipelines (created_at DESC);
CREATE INDEX pipelines_project_id_idx ON pipelines (project_id);

-- ── Triggers ──────────────────────────────────────────────────────────────────
-- Stored initiators that launch pipeline runs (cron schedule or inbound webhook).
-- `source`/`inputs` are serialized value objects; `kind` is the denormalized
-- TriggerSource discriminant kept for the engine's due-scan / routing. Firing
-- flows through the normal run path — a trigger is a new source, not a new
-- execution path.
CREATE TABLE pipeline_triggers (
    id            TEXT        PRIMARY KEY,
    pipeline_id   TEXT        NOT NULL REFERENCES pipelines (id) ON DELETE CASCADE,
    name          TEXT        NOT NULL,
    kind          TEXT        NOT NULL CHECK (kind IN ('cron', 'webhook')),
    source        JSONB       NOT NULL,
    inputs        JSONB       NOT NULL DEFAULT '[]',
    enabled       BOOLEAN     NOT NULL DEFAULT TRUE,
    next_fire_at  TIMESTAMPTZ,
    last_fired_at TIMESTAMPTZ,
    last_status   TEXT,
    -- Webhook triggers only: the HMAC signing secret, AEAD-encrypted (nonce||ct)
    -- with the control-plane master key. NULL for cron. Verification needs the
    -- plaintext, so it is encrypted-reversible, never one-way hashed. Never leaves
    -- the server except through the ingress verify path.
    webhook_secret_enc BYTEA,
    created_at    TIMESTAMPTZ NOT NULL,
    updated_at    TIMESTAMPTZ NOT NULL,
    UNIQUE (pipeline_id, name)
);
CREATE INDEX pipeline_triggers_pipeline_id_idx ON pipeline_triggers (pipeline_id);
-- Engine due-scan (cron): tiny partial index, shaped for FOR UPDATE SKIP LOCKED.
CREATE INDEX pipeline_triggers_due_idx ON pipeline_triggers (next_fire_at)
    WHERE enabled AND next_fire_at IS NOT NULL;

-- Inbound webhook deliveries, for replay dedupe. A delivery is identified by the
-- sender's id (or, absent one, the request signature); a repeat (trigger_id,
-- delivery_id) is a replay and is accepted but not re-fired. Cascade-deleted with
-- the trigger.
CREATE TABLE trigger_deliveries (
    trigger_id  TEXT        NOT NULL REFERENCES pipeline_triggers (id) ON DELETE CASCADE,
    delivery_id TEXT        NOT NULL,
    received_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (trigger_id, delivery_id)
);

-- ── Project secrets ───────────────────────────────────────────────────────────
-- Reversible-encrypted values referenced from pipeline node env vars. The value
-- is AEAD ciphertext (nonce || ciphertext || tag); plaintext is decrypted only
-- at dispatch time and never returned by the API.
CREATE TABLE project_secrets (
    id              TEXT        PRIMARY KEY,
    project_id      TEXT        NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    description     TEXT        NOT NULL DEFAULT '',
    encrypted_value BYTEA       NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL,
    UNIQUE (project_id, name)
);
CREATE INDEX project_secrets_project_id_idx ON project_secrets (project_id);

-- ── Apps (machine principals) & agents ────────────────────────────────────────

CREATE TABLE apps (
    id              TEXT        PRIMARY KEY,
    organization_id TEXT        NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    is_active       BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (organization_id, name)
);
CREATE INDEX apps_organization_id_idx ON apps (organization_id);

CREATE TABLE app_secrets (
    id          TEXT        PRIMARY KEY,
    app_id      TEXT        NOT NULL REFERENCES apps (id) ON DELETE CASCADE,
    label       TEXT        NOT NULL,
    secret_hash TEXT        NOT NULL,
    enabled     BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (app_id, label)
);
CREATE INDEX app_secrets_app_id_idx ON app_secrets (app_id);

CREATE TABLE app_tokens (
    id         TEXT        PRIMARY KEY,
    token      TEXT        NOT NULL,
    app_id     TEXT        NOT NULL REFERENCES apps (id) ON DELETE CASCADE,
    secret_id  TEXT        NOT NULL REFERENCES app_secrets (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);
CREATE UNIQUE INDEX app_tokens_token_key ON app_tokens (token);
CREATE INDEX app_tokens_app_id_idx ON app_tokens (app_id);
CREATE INDEX app_tokens_secret_id_idx ON app_tokens (secret_id);
CREATE INDEX app_tokens_expires_at_idx ON app_tokens (expires_at);

CREATE TABLE agents (
    app_id     TEXT        PRIMARY KEY REFERENCES apps (id) ON DELETE CASCADE,
    last_seen  TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Jobs & logs ──────────────────────────────────────────────────────────────

CREATE TABLE jobs (
    id              TEXT        PRIMARY KEY,
    pipeline_id     TEXT        NOT NULL REFERENCES pipelines (id) ON DELETE CASCADE,
    status          TEXT        NOT NULL,
    node_executions JSONB       NOT NULL,
    -- Trigger-supplied literal env overlaid on every node at dispatch (e.g.
    -- GIT_COMMIT). Persisted so the run dispatches identically whether placed
    -- immediately or retried later by the pending-job scheduler. Literals only —
    -- never secrets.
    inputs          JSONB       NOT NULL DEFAULT '[]',
    agent_app_id    TEXT        REFERENCES agents (app_id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL,
    started_at      TIMESTAMPTZ,
    finished_at     TIMESTAMPTZ
);
CREATE INDEX jobs_pipeline_id_idx ON jobs (pipeline_id);
CREATE INDEX jobs_status_idx ON jobs (status);
CREATE INDEX jobs_created_at_idx ON jobs (created_at DESC);
CREATE INDEX jobs_agent_app_id_created_at_idx ON jobs (agent_app_id, created_at DESC);

CREATE TABLE job_logs (
    id         TEXT        PRIMARY KEY,
    job_id     TEXT        NOT NULL REFERENCES jobs (id) ON DELETE CASCADE,
    node_id    TEXT        NOT NULL,
    stream     TEXT        NOT NULL,
    line       TEXT        NOT NULL,
    "timestamp" TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX job_logs_job_id_timestamp_idx ON job_logs (job_id, "timestamp");
CREATE INDEX job_logs_job_id_node_id_timestamp_idx ON job_logs (job_id, node_id, "timestamp");

-- ── Invitations ──────────────────────────────────────────────────────────────

CREATE TABLE organization_invites (
    id              TEXT        PRIMARY KEY,
    organization_id TEXT        NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    email           TEXT        NOT NULL,
    role_name       TEXT,
    token           TEXT        NOT NULL UNIQUE,
    status          TEXT        NOT NULL DEFAULT 'pending',
    invited_by      TEXT        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    expires_at      TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL
);
CREATE INDEX organization_invites_org_idx ON organization_invites (organization_id);
CREATE INDEX organization_invites_token_idx ON organization_invites (token);

-- ── Authorization: audit, policies, roles, grants ────────────────────────────

CREATE TABLE audit_log (
    id             TEXT        PRIMARY KEY,
    occurred_at    TIMESTAMPTZ NOT NULL,
    principal_kind TEXT        NOT NULL,
    principal_id   TEXT,
    action         TEXT        NOT NULL,
    resource_kind  TEXT        NOT NULL,
    resource_id    TEXT,
    decision       TEXT        NOT NULL,
    policies       TEXT[]      NOT NULL DEFAULT '{}',
    reason         TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX audit_log_occurred_at_idx ON audit_log (occurred_at);
CREATE INDEX audit_log_principal_idx ON audit_log (principal_kind, principal_id);
CREATE INDEX audit_log_resource_idx ON audit_log (resource_kind, resource_id);
CREATE INDEX audit_log_action_idx ON audit_log (action);

-- Advanced Cedar escape-hatch rules, layered on top of the role/grant-generated
-- policy set. Validated on write, hot-reloaded into the live authorizer.
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

-- A role: a named, editable bundle of permissions bound to a scope kind. The
-- Cedar policy set is generated from these rows. Builtin roles have a stable
-- key (== id); tenant custom roles set owner_org_id.
CREATE TABLE roles (
    id           TEXT        PRIMARY KEY,
    key          TEXT        UNIQUE,
    name         TEXT        NOT NULL,
    description  TEXT        NOT NULL DEFAULT '',
    scope_kind   TEXT        NOT NULL CHECK (scope_kind IN ('system', 'organization', 'project')),
    owner_org_id TEXT        REFERENCES organizations (id) ON DELETE CASCADE,
    builtin      BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX roles_owner_org_idx ON roles (owner_org_id);

-- A role's permissions: each row is a `Permission::key()` (e.g. 'runPipeline'),
-- or the sentinel '*' meaning full control within the grant's scope.
CREATE TABLE role_permissions (
    role_id    TEXT NOT NULL REFERENCES roles (id) ON DELETE CASCADE,
    permission TEXT NOT NULL,
    PRIMARY KEY (role_id, permission)
);

-- An explicit scoped grant: "principal P holds TARGET within scope S", where the
-- target is either a whole role or a single permission (additive to roles). The
-- one authorization mechanism: role grants link a Cedar role-template instance,
-- permission grants generate a direct permit policy.
CREATE TABLE grants (
    id             TEXT        PRIMARY KEY,
    principal_kind TEXT        NOT NULL CHECK (principal_kind IN ('user', 'app')),
    principal_id   TEXT        NOT NULL,
    target_kind    TEXT        NOT NULL CHECK (target_kind IN ('role', 'permission')),
    -- The role id (target_kind='role') or a Permission::key (target_kind='permission').
    target         TEXT        NOT NULL,
    scope_kind     TEXT        NOT NULL CHECK (scope_kind IN ('system', 'organization', 'project')),
    scope_id       TEXT        NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (principal_kind, principal_id, target_kind, target, scope_kind, scope_id)
);
CREATE INDEX grants_principal_idx ON grants (principal_kind, principal_id);

-- Configurable pointer from a "default role" slot (e.g. the role assigned to the
-- creator on org/project creation) to a role. Decouples the creation flows from
-- a hard-coded role name: an admin can rebind a slot to a custom role.
-- ON DELETE RESTRICT keeps a bound role from being deleted out from under the
-- slot, so the pointer never dangles (rebind first, then delete).
CREATE TABLE default_role_bindings (
    -- Mirror of DefaultRoleSlot::as_str — a bad slot would silently never
    -- resolve, so reject it at write time.
    slot    TEXT PRIMARY KEY CHECK (slot IN ('org_creation', 'project_creation')),
    role_id TEXT NOT NULL REFERENCES roles (id) ON DELETE RESTRICT
);

-- ── Seed: builtin roles ───────────────────────────────────────────────────────
-- Admin roles confer full control ('*'); agent roles only the job-execution
-- permissions. The Cedar policy set is generated from these.
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
    ('organization-agent', 'appendJobLog'),
    ('project-agent',      'readPipeline'),
    ('project-agent',      'executeJob'),
    ('project-agent',      'writeJobStatus'),
    ('project-agent',      'appendJobLog');

-- ── Seed: default-role slots ──────────────────────────────────────────────────
-- Point each creation flow at its builtin admin role by default; rebindable.
INSERT INTO default_role_bindings (slot, role_id) VALUES
    ('org_creation',     'organization-admin'),
    ('project_creation', 'project-admin');
