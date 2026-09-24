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
```

**This module is Svelte** (Phase 3). The two hooks that used to be exported are gone: they
were view models, not API, and nothing outside this feature ever called them. What is left is
the pure domain, which is the only part another module would want.

Both pages are **deliberately not exported**: `membership.module.ts` loads them lazily, and a
barrel that re-exported them would pull them into the chunk of anything importing this module.
Never add: `membership.module.ts`, pages.

## Data contract

**This module owns no repository.** `domain` is `{}`; it is registered for its routes alone.
There is no `use-membership-domain.ts`, and adding one would be wrong.

It composes, through public APIs only:

| From | Used for |
|---|---|
| `features/roles` | `roleQueries`, `grantMutations`, `roleConfers`, `humanizeRoleId`, `scopeLabelOf`, `RoleEntity` |
| `features/organization` | organization member list |
| `features/project` | project member list |
| `features/user` | user identities |
| `@platform/authz` | `PrincipalKind`, `PermissionScope`, `Permission`, `can` |

**This module reads the grants through `roleQueries.scopedGrants`**, the same factory that
`roles` uses — so one resource has exactly one query key.

## Layout

```
membership.module.ts                 routes + nav, empty domain
index.ts                             public API
domain/structs/scope-member.struct.ts   ScopeMember, MemberRole, MemberRoleOrigin +
                                        buildOrganizationMembers / buildProjectMembers (pure)
presentation/
  scope-membership.state.svelte.ts   grants → members, plus grant/revoke (a ViewModel)
  assignable-roles.state.svelte.ts   which roles *this* user may hand out
  ui/OrganizationMembers.page.svelte, ProjectMembers.page.svelte
  ui/membership.messages.ts          every string — `lingui extract` cannot read `.svelte`
  ui/components/                     AddMemberDialog (+ AddMemberForm), AddRoleSelect,
                                     MemberCard, MemberIdentity, MemberRoleBadges,
                                     MemberRowAction, MembersHint, MembersList, RoleChecklist
  ui/components/index.ts             local barrel — internal, not the module's public API
```

Both ViewModels take **getters**, not values, for anything that arrives late — the scope id
from the route or the context store, the permission from a store that loads after first paint.
Capturing either once freezes the view on the first, usually empty, render. The React hooks
re-ran on every render and got this for free.

No `infrastructure/` — correct, do not add one.

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `organization` | `members` | `LIST_ORGANIZATION_MEMBERS` | `OrganizationMembers.page.svelte` |
| `project` | `members` | `LIST_PROJECT_MEMBERS` | `ProjectMembers.page.svelte` |

The project page takes `projectId` as a **prop**: route params are the one thing a Svelte page
cannot read from a singleton, so the router gives them as props (`@platform/routing`).

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
- **New strings go in `ui/membership.messages.ts`, never inside a `.svelte`.** Extraction does
  not read components, so a message declared there vanishes from the catalogs without failing a
  gate.
- **The role-count plural is load-bearing and has a French test.** `<Plural _0=…>` became
  `plural(value, { 0: … })`; the two compile to the same `=0 {…}` arm, which is what kept the
  translation attached. `MemberCard.fr.test.ts` is what proves it — no gate would have.
- `AddMemberDialog` splits into a dialog and an `AddMemberForm` on purpose: the form owns the
  selection, so `{#key open}` on it *is* the reset React needed an effect for.
- A bits-ui select's options live in a floating layer jsdom hides. Reach them with
  `findFloating('option', …)` from `src/test/render.svelte.ts`, never `getByText`.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
