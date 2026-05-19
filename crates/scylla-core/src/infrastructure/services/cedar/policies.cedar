// scylla — Cedar authorization policies (V1).
//
// Defaults to deny: only the explicit `permit` statements below grant access.
// Policies are static and compiled into the binary via `include_str!`.

// ── Admin role ────────────────────────────────────────────────────────
// Any user member of Scylla::Role::"admin" has full access on every
// (action, resource) pair. Membership is materialised at check time from
// the `user_roles` table by the entity provider in CedarPermissionService.
permit (
    principal in Scylla::Role::"admin",
    action,
    resource
);

// ── Internal services ────────────────────────────────────────────────
// In-process subsystems (recorder, etc.) bypass user-level RBAC. This is
// the explicit, auditable replacement for the silent recorder bypass that
// existed before zero-trust prep landed.
//
// Tighten later (per-service action allowlists, principal-id pinning) once
// service-to-service mTLS / SPIFFE identities are in place.
permit (
    principal,
    action,
    resource
) when {
    principal is Scylla::Service
};
