-- Standardize role names to the `<scope>-<role>` convention (system / organization
-- / project). Rewrites the names stored before this convention existed:
--   - global `admin`  -> `system-admin`   (user_roles; matched by the Cedar
--     `@id("admin")` policy which now targets Scylla::Role::"system-admin")
--   - scoped `agent`  -> `organization-agent` (permission_grants; agents are
--     minted on an organization scope)
-- Idempotent: re-running (or running on a fresh DB where the bootstrap already
-- writes the new names) matches no rows. Membership tiers stay ABAC, so there is
-- nothing to rewrite for `*-member`.

UPDATE user_roles
SET role_name = 'system-admin'
WHERE role_name = 'admin';

UPDATE permission_grants
SET role_name = 'organization-agent'
WHERE role_name = 'agent';
