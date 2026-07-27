// scylla — Cedar authorization policies (V1). Default-deny; only the explicit
// permits below grant access. Compiled into the binary via include_str! and
// validated against schema.cedarschema at startup.

// ── System admin ────────────────────────────────────────────────────────
// There is no static admin policy: `system-admin` is now a GRANT on the System
// scope (the tenancy root), linked as a role-template instance like any other
// grant. Because every resource is `in` System, that single grant confers full
// control over everything. See `application/permission/grant.rs` (SYSTEM scope).

// ── Internal services ──────────────────────────────────────────────────
// In-process subsystems (recorder, bootstrap) bypass user RBAC. Explicit,
// auditable replacement for the old silent recorder bypass. Tighten later to
// per-service action allowlists once service identities are on the wire.
@id("service")
permit (
    principal,
    action,
    resource
) when {
    principal is Scylla::Service
};

// ── Self access (ABAC) ─────────────────────────────────────────────────
// A user owns its own User entity: it may read and update its own profile and
// list the orgs/projects it belongs to. `deleteUser` is intentionally excluded
// — self-deletion is destructive and stays an admin / dedicated-flow action.
// Without this, only global admins could fetch their own org/project list —
// non-admins were denied listUserOrganizations even for themselves.
@id("self")
permit (
    principal,
    action in [
        Scylla::Action::"readUser",
        Scylla::Action::"updateUser",
        Scylla::Action::"listUserOrganizations",
        Scylla::Action::"listUserProjects"
    ],
    resource
) when {
    principal is Scylla::User && principal == resource
};

// ── Organization membership (ABAC) ─────────────────────────────────────
// The one thing membership confers, and deliberately the only one: you may see
// the organization you belong to, and who else is in it. Everything else comes
// from a grant (see docs/src/access-model.md). Without this a member would need
// a grant merely to see their own organization, which reads as a bug rather
// than as a security boundary.
//
// Note what is NOT here: listing an organization's projects. That is what makes
// "in an organization without seeing all its projects" expressible — the project
// list is filtered by the grants you actually hold.
@id("org-member")
permit (
    principal,
    action in [
        Scylla::Action::"readOrganization",
        Scylla::Action::"listOrganizationMembers"
    ],
    resource
) when {
    principal is Scylla::User && resource in principal.memberOrgs
};

// There is no project counterpart to the policy above. Being on a project means
// holding a role on it, so a project confers nothing on its own; what you may do
// there is exactly what your role says.
