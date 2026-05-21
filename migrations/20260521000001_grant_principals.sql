-- Generalize permission_grants from user-only to any principal (user or app).
-- A grant now binds a (principal_kind, principal_id) pair to a scoped role, so a
-- machine App can hold the same scoped roles as a User. The users foreign key is
-- dropped because an app principal is not a row in `users`; referential
-- integrity for the principal is enforced in the application layer instead.

ALTER TABLE permission_grants DROP CONSTRAINT IF EXISTS permission_grants_user_id_fkey;
ALTER TABLE permission_grants
    DROP CONSTRAINT IF EXISTS permission_grants_user_id_role_name_scope_kind_scope_id_key;
DROP INDEX IF EXISTS permission_grants_user_id_idx;

ALTER TABLE permission_grants RENAME COLUMN user_id TO principal_id;
ALTER TABLE permission_grants ADD COLUMN principal_kind TEXT NOT NULL DEFAULT 'user';
ALTER TABLE permission_grants ALTER COLUMN principal_kind DROP DEFAULT;
ALTER TABLE permission_grants
    ADD CONSTRAINT permission_grants_principal_kind_check
    CHECK (principal_kind IN ('user', 'app'));

ALTER TABLE permission_grants
    ADD CONSTRAINT permission_grants_principal_role_scope_key
    UNIQUE (principal_kind, principal_id, role_name, scope_kind, scope_id);
CREATE INDEX permission_grants_principal_idx
    ON permission_grants (principal_kind, principal_id);
