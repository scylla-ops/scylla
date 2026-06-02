-- Unify the two authorization mechanisms into one. A "global role" is no longer
-- a row in `user_roles`; it is a grant on the new System scope (the tenancy
-- root). `RoleService` is removed — everything goes through `permission_grants`.

-- 1. Allow the System scope on grants.
ALTER TABLE permission_grants DROP CONSTRAINT IF EXISTS permission_grants_scope_kind_check;
ALTER TABLE permission_grants
    ADD CONSTRAINT permission_grants_scope_kind_check
    CHECK (scope_kind IN ('system', 'organization', 'project'));

-- 2. Migrate every existing global role (e.g. system-admin) to a System-scoped
--    user grant. `scope_id = 'system'` is the singleton-root sentinel (the app
--    ignores it on read). Idempotent via the principal/role/scope unique key.
INSERT INTO permission_grants (id, principal_kind, principal_id, role_name, scope_kind, scope_id)
SELECT gen_random_uuid()::text, 'user', user_id, role_name, 'system', 'system'
FROM user_roles
ON CONFLICT (principal_kind, principal_id, role_name, scope_kind, scope_id) DO NOTHING;

-- 3. The role table is now obsolete.
DROP TABLE user_roles;
