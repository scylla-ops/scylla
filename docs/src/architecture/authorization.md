# Authorization model

Scylla's authorization is **RBAC generated into
[Cedar](https://www.cedarpolicy.com/) policies, over an ABAC base**. This is
the deep dive; the user-facing view is
[Users, orgs & access](../using/access.md).

## The vocabulary (one word per concept)

These words are a single source of truth — the same in code, proto, DB, and
UI:

- **Permission** — an atomic capability (a verb on a resource type, e.g.
  `runPipeline`).
- **Role** — a named bundle of permissions bound to a scope kind.
- **Scope** — the level a grant binds to: `System`, `Organization(id)`, or
  `Project(id)`.
- **Grant** — *principal P holds {a role | a permission} within scope S*.
- **Principal** — a grant-holding actor: a `User` or an `App`.
- **Caller** — the request identity (`User`, `App`, `Service`, or
  `Anonymous`).
- **Policy** — an advanced Cedar escape-hatch rule.
- **Resource** — the entity an action targets.

## Scopes form a tree

`System ⊃ Organization ⊃ Project`. `System` is the tenancy root:
`Organization` and `User` are `in [System]`, and `Project`/`Pipeline`/`Job`
sit beneath their org. A grant at a scope reaches everything in its subtree —
`system-admin` covers everything, an org grant covers that org's projects. A
role can only carry a permission whose target resource lives within the
role's scope subtree — the catalog rejects e.g. `createOrganization` (targets
`system`) in a project role.

## The grant is the one mechanism

There is exactly **one** authorization primitive: the grant, a
`(principal, target, scope)` row in the `grants` table where target is a role
or a single permission. A "global role" is just a grant on the `System` scope
— there is no separate user-roles table. A direct permission grant is
**additive** to a principal's role-derived permissions.

The [builtin roles](../using/access.md#scopes-roles--grants) are seeded on
first boot. The implicit *member* tiers (`system-member`,
`organization-member`, `project-member`) are **not** stored roles —
membership is ABAC via the `user_organization` / `user_project` tables.

Custom roles (a chosen permission set, org-owned) can be created at runtime;
**permissions cannot** — the `Permission` catalog is a closed, code-owned
enum, because only the code that enforces a capability gives it meaning.

## Cedar policy generation

The live Cedar policy set is **generated from the RBAC rows**, not
hand-written:

- Each **role** becomes a Cedar policy body — a full-control role (`*`) maps
  to the unconstrained-action body; any other role lists its permission keys.
- Each **role grant** is linked as a Cedar template instance binding the
  principal and the scope (`resource in ?resource`), confining it to its
  subtree.
- A **static ABAC base** (`policies.cedar`) covers membership and self-read;
  the admin-defined `cedar_policies` table adds escape-hatch rules.

Grants and role edits are applied **live** via a policy reload — no
control-plane restart. Revoking an App's grant also disconnects its agent
stream, so a no-longer-authorized worker stops immediately.

## Enforcement: `PermissionService`

Every use case authorizes through one port:

```rust
check(caller, Permission) -> DomainResult<()>
```

It returns `Ok(())` or `Err(Forbidden)` — **never `Ok(false)`**. Using a
`Result` instead of a `bool` makes the check **fail-closed by construction**:
a caller can only proceed by handling the error, so a denial can't be
silently treated as success. The production adapter,
`CedarPermissionService`, builds the principal/resource entities, snapshots
the live policy set, asks Cedar, and records an audit-log row for the
decision.

## Guard-rails

The grant use cases enforce invariants Cedar alone wouldn't:

- **Anti-escalation** — a delegator may only confer permissions it already
  holds at that scope (checked for System and Organization grants), so a
  narrow "manage grants" holder can't grant itself full admin.
- **Last-owner protection** — a scope must keep at least one *human* owner;
  revoking the final `*-admin` user grant is refused.
- **Scope-pinned permission families** — `manage{System,Org,Project}Grants`
  and the `list*By{Organization,Project,Pipeline}` families are deliberately
  distinct Cedar actions (an anti-escalation fence); the UI presents each
  family as one concept with a scope selector, but the split is load-bearing
  in the schema.
