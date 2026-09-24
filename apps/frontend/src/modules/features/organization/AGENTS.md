# `features/organization` — agent guide

Organizations: the top-level tenant, its members, and the switcher in the shell.

**Layer** `features/` · **id** `organization` · **DI key** `organizationRepository`

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

**Presentation is Svelte** (Phase 2 of `refacto_svelte.md`). Domain and infrastructure are
unchanged. There is no `use-<feature>-domain.ts` and no hooks: reads and writes are declared as
options objects in `presentation/organization.queries.ts`, which a component or another
feature runs with `createQuery`.

**`OrganizationList` is the one list of organizations, for two places.** The user settings panel
shows it with plain rows (`OrganizationRow.svelte`). The organization selector of the shell shows
it inside a dropdown menu, and gives it `DropdownMenuItem` as the `row` prop, so each row gets
the keyboard focus of the menu. A row component takes `class`, `onSelect` and `children`.

## Public API — `index.ts`

```typescript
type OrganizationEntity
organizationQueries        mine · members
organizationMutations      create · update · remove
invalidateOrganizationMembers
ORGANIZATIONS_QUERY_KEY, MY_ORGANIZATIONS_QUERY_KEY, ORGANIZATION_MEMBERS_QUERY_KEY
createOrganizationItems
loadOrganizationList         () => import(OrganizationList.svelte)       ← the shell's selector
loadAddOrganizationDialog    () => import(AddOrganizationDialog.svelte)  ← the shell's selector
```

The shell imports this barrel eagerly for the queries, so the components are exported as
**loaders**: a re-exported component would put its UI library in the entry chunk. Never add:
`organization.module.ts`, `UserSettingsRoute`, a component.

## Data contract

`OrganizationRepository` — `domain/repository/organization.repository.ts` (**default** export):

| Method | Returns |
|---|---|
| `getAll()` | `OrganizationEntity[]` — every org (admin view) |
| `getMine()` | `OrganizationEntity[]` — the current user's orgs |
| `listMembers(organizationId)` | `UserEntity[]` |

`getAll` vs `getMine` is a real distinction — the switcher must use `getMine`. Reach the
repository with `getModuleDomain` **inside `organization.queries.ts` only**.

## Layout

```
organization.module.ts               route + DI wiring (private; registry only)
index.ts                             public API
domain/
  entities/organization.entity.ts    OrganizationEntity
  repository/organization.repository.ts
infrastructure/
  data/grpc-organization-remote.data-source.ts          impl
  repository/data-sources/organization-remote.data-source.ts   interface
  repository/mappers/grpc-organization.mapper.ts
  repository/mappers/grpc-organization-member.mapper.ts
  repository/default-organization.repository.ts
presentation/
  organization.queries.ts            every read and write, plus the key factories
  ui/OrganizationList.svelte         plain rows, for the settings panel
  ui/AddOrganizationDialog.svelte, EditOrganizationDialog.svelte
  ui/UserSettingsRoute.svelte        composes user's UserSettingsPage
  ui/organization.messages.ts
  utils/create-organization-form-items.ts
```

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `organization` | `users/:userId` | none declared | `UserSettingsRoute` |

**No nav entry**, and the route looks misplaced on purpose. The user directory belongs to
[`user`](../user/AGENTS.md), which owns the `users` page and crumb; this module declares
`users/:userId` because the settings page renders an organizations panel. The router joins the
two by path: this page shows the "Users" crumb of `user`, and neither module imports the other.

## Rules that bite here

- **`UserSettingsRoute` is a composition seam.** It renders `UserSettingsPage`, imported from
  `features/user`'s public API — one of the two sanctioned page exports in the codebase. Keep
  the wrapper thin; do not copy user logic into it.
- `ORGANIZATION_MEMBERS_QUERY_KEY` is exported so `membership` invalidates the same entry this
  module reads. Never hand-write the key.
- The shell depends on `OrganizationList` / `AddOrganizationDialog`. Changing their props is a
  breaking change for `layout/` — update `OrganizationSelector` in the same commit.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
