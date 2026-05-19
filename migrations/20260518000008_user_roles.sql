-- Replaces the previous Casbin-backed RBAC table.
--
-- Each row asserts that `user_id` belongs to `role_name`. Membership is read
-- at check time by `CedarPermissionService` to build the principal entity's
-- `parents` set (e.g. `Scylla::User::"<id>"` in `Scylla::Role::"admin"`).
--
-- Roles are referenced by name (free-form string). The bootstrap admin user
-- gets the `admin` role on first boot. Future roles (auditor, viewer, ...)
-- can be added by INSERTing rows here without schema changes.

CREATE TABLE user_roles (
    user_id    TEXT        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_name  TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, role_name)
);

CREATE INDEX user_roles_role_name_idx ON user_roles (role_name);
