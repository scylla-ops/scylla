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

// ── Organization members (ABAC) ────────────────────────────────────────
// A user may perform read/list operations on an organization it belongs to.
// `principal is User` narrows the type so `principal.memberOrgs` typechecks.
@id("org-member")
permit (
    principal,
    action in [
        Scylla::Action::"readOrganization",
        Scylla::Action::"listOrganizationMembers",
        Scylla::Action::"listProjectsByOrganization",
        Scylla::Action::"listPipelinesByOrganization",
        Scylla::Action::"listJobsByOrganization"
    ],
    resource
) when {
    principal is Scylla::User && resource in principal.memberOrgs
};

// ── Project members (ABAC) ─────────────────────────────────────────────
// A user may operate within a project it belongs to. Resources below a member
// project (pipelines, jobs) match via the entity hierarchy.
@id("project-member")
permit (
    principal,
    action in [
        Scylla::Action::"readProject",
        Scylla::Action::"listProjectMembers",
        Scylla::Action::"createPipeline",
        Scylla::Action::"readPipeline",
        Scylla::Action::"updatePipeline",
        Scylla::Action::"deletePipeline",
        Scylla::Action::"runPipeline",
        Scylla::Action::"listPipelinesByProject",
        Scylla::Action::"readJob",
        Scylla::Action::"listJobsByProject",
        Scylla::Action::"listJobsByPipeline",
        Scylla::Action::"readJobLogs"
    ],
    resource
) when {
    principal is Scylla::User && resource in principal.memberProjects
};
