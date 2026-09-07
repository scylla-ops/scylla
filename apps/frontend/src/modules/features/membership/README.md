# Membership

> [Scylla frontend](../../../../README.md) › `features/` › **membership** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

Who belongs to an organization or a project, with which roles, and how to change that. Two
pages — `/:org/members` and `/:org/projects/:projectId/members` — showing the same idea about
two different subjects.

## Why membership is its own module

It could have been a corner of [organization](../organization/README.md) and a corner of
[project](../project/README.md). It isn't, for two reasons.

First, both scopes answer the *same question* with the same components: a list of principals,
their roles, an add dialog, a revoke action. Splitting it in two would have meant maintaining
that twice and letting the copies drift.

Second, membership is not a thing either module stores. Neither organizations nor projects have
a member list in the backend — **membership is a derived view of grants**. Putting the derivation
inside `organization` would have made that module depend on `roles`, and `project` too, for a
concern belonging to neither.

So `membership` sits above both and composes four modules through their public APIs — `roles`,
`organization`, `project`, `user` — while nothing except the router depends on it. It owns no
repository at all: its `domain` is `{}` and there is no `infrastructure/` folder.

## Members are computed from grants

The core of the module is `scope-member.struct.ts`, and it is pure logic:
`buildOrganizationMembers` and `buildProjectMembers` take grants and user identities and fold
them into `ScopeMember[]` — one entry per principal, each carrying the roles that principal
holds.

Because it lives in `domain/` as plain functions, the same rules apply on both pages, and
"who is a member" has exactly one definition.

### Direct vs inherited

The distinction that matters most is `MemberRoleOrigin`:

- **`DIRECT`** — the role was granted at this exact scope.
- **`INHERITED`** — the role was granted at the organization and therefore applies inside the
  project too.

A project page shows both, because a user with an org-level role really can act on the project.
But only direct roles can be revoked from there; removing an inherited one means going up to the
organization. The UI reflects this — `MemberRoleBadges` distinguishes the two, and
`MemberRowAction` only offers revocation where it would work. Flattening the distinction would
produce a revoke button that silently does nothing.

## You can only grant what you may grant

`useAssignableRoles` answers a narrower question than "which roles exist": which roles is the
*current user* permitted to hand out, at this scope. It combines the role catalog, the grantable
list from the backend, and the user's own permissions. The add-member dialog is built from that,
so a user is never offered a role the server would refuse.

## Structure at a glance

- `domain/structs/scope-member.struct.ts` — the whole domain: types and the two builders.
- `presentation/hooks/use-scope-membership.ts` — grants for a scope, plus grant and revoke
  mutations, with toasts.
- `presentation/hooks/use-assignable-roles.ts` — the assignable-role calculation.
- `presentation/ui/` — two pages over one set of scope-agnostic components.

Neither page is exported from `index.ts`. They are lazily loaded by the module declaration, and
exporting them would drag them into the bundle of anything importing this module's hooks.

## Related modules

- [roles](../roles/README.md) — grants, the role catalog, and the permission vocabulary.
- [organization](../organization/README.md) and [project](../project/README.md) — the two scopes.
- [user](../user/README.md) — the human principals behind the members.
- [platform/authz](../../platform/authz/README.md) — `PrincipalKind`, `PermissionScope`, gating.
