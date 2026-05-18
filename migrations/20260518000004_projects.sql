-- Projects belong to an organization; deleting the org cascades. The N:M
-- membership table mirrors `user_organization` (same shape, same cascade).

CREATE TABLE projects (
    id              TEXT        PRIMARY KEY,
    name            TEXT        NOT NULL,
    description     TEXT        NULL,
    organization_id TEXT        NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    is_active       BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL
);

CREATE INDEX projects_organization_id_idx ON projects (organization_id);
CREATE INDEX projects_created_at_idx ON projects (created_at DESC);

CREATE TABLE user_project (
    user_id         TEXT        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    project_id      TEXT        NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, project_id)
);

CREATE INDEX user_project_project_id_idx ON user_project (project_id);
