# `features/membership` — agent guide

Who belongs to an organization or a project, and with which roles.

**Layer** `features/` · **id** `membership` · **DI key** none (`domain: {}`)

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

## Public API — `index.ts`

```typescript
type ScopeMember, MemberRole
MemberRoleOrigin                      (enum — a value, not a type)
buildOrganizationMembers, buildProjectMembers
useScopeMembership
useAssignableRoles, type AssignableRole
```

Both pages are **deliberately not exported**: `membership.module.ts` loads them lazily, and a
barrel that re-exported them would pull them into the chunk of anything importing this module.
Never add: `membership.module.ts`, pages.

## Data contract

**This module owns no repository.** `domain` is `{}`; it is registered for its routes alone.
There is no `use-membership-domain.ts`, and adding one would be wrong.

It composes, through public APIs only:

| From | Used for |
|---|---|
| `features/roles` | `useScopedGrants`, `useGrantableRoles`, `useRoles`, `humanizeRoleId`, `RoleEntity` |
| `features/organization` | organization member list |
| `features/project` | project member list |
| `features/user` | user identities |
| `@platform/authz` | `PrincipalKind`, `PermissionScope`, `Permission`, `useCan` |

## Layout

```
membership.module.ts                 routes + nav, empty domain
index.ts                             public API
domain/structs/scope-member.struct.ts   ScopeMember, MemberRole, MemberRoleOrigin +
                                        buildOrganizationMembers / buildProjectMembers (pure)
presentation/
  hooks/use-scope-membership.ts      grants → members, plus grant/revoke
  hooks/use-assignable-roles.ts      which roles *this* user may hand out
  ui/OrganizationMembers.page.tsx, ProjectMembers.page.tsx
  ui/components/                     AddMemberDialog, AddRoleSelect, MemberCard,
                                     MemberIdentity, MemberRoleBadges, MemberRowAction,
                                     MembersHint, MembersList, RoleChecklist
  ui/components/index.ts             local barrel — internal, not the module's public API
```

No `infrastructure/` — correct, do not add one.

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `organization` | `members` | `LIST_ORGANIZATION_MEMBERS` | `OrganizationMembersPage` |
| `project` | `members` | `LIST_PROJECT_MEMBERS` | `ProjectMembersPage` |

Sidebar: one entry only — section `organization`, order `30`, icon `UsersRound`, gated on
`LIST_ORGANIZATION_MEMBERS`. The project page is reached from inside a project, not the org nav.

## Rules that bite here

- **A member is a derived view of grants, not a stored list.** `buildOrganizationMembers` and
  `buildProjectMembers` fold grants + identities into `ScopeMember[]`. They are pure functions
  in `domain/` — put membership logic there, not in a component or a `useMemo`.
- **`MemberRoleOrigin` is the load-bearing distinction.** `DIRECT` = granted at this scope;
  `INHERITED` = granted at the organization and visible on the project. Only `DIRECT` roles are
  revocable here — revoking an inherited role means going to the organization. Never render the
  two identically.
- `useAssignableRoles` exists because you may only grant roles you are allowed to grant. Do not
  bypass it and list every role from `useRoles`.
- Two pages, one shape: keep the shared components scope-agnostic and pass the scope in. Do not
  fork `MembersList` per scope.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles` — all clean.
New strings: `pnpm extract && pnpm compile`.
