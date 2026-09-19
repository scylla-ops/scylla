# `features/organization` — agent guide

Organizations: the top-level tenant, its members, and the switcher in the shell.

**Layer** `features/` · **id** `organization` · **DI key** `organizationRepository`

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

**Presentation is Svelte** (Phase 2 of `refacto_svelte.md`). Domain and infrastructure are
unchanged. There is no `use-<feature>-domain.ts` and no hooks: reads and writes are declared as
options objects in `presentation/organization.queries.ts`, which both bindings can run —
`createQuery` here, react-query's `useQuery` in the modules still on React.

**One piece of this module's UI moved out rather than across.** The organization switcher's list
used to live here and take its row wrapper as a component prop, so the sidebar could make each
row a `DropdownMenuItem`. A Svelte component cannot be handed a React one — Radix's menu item
provides roving focus and `onSelect` through React context — so the rendering moved to
`layout/presentation/ui/context-selector/OrganizationSwitcherList.tsx`, next to the dropdown it
belongs to, and reads this module's queries through the barrel. `OrganizationList.svelte` here
renders the same list as plain blocks for the user settings panel. The two meet again in Phase 6,
when the sidebar becomes Svelte and the React copy goes.

## Public API — `index.ts`

```typescript
type OrganizationEntity
organizationQueries        mine · members
organizationMutations      create · update · remove
invalidateOrganizationMembers
ORGANIZATIONS_QUERY_KEY, MY_ORGANIZATIONS_QUERY_KEY, ORGANIZATION_MEMBERS_QUERY_KEY
createOrganizationItems
OrganizationList                              ← the settings panel (Svelte)
AddOrganizationDialog, EditOrganizationDialog ← mounted as islands by the React shell
```

The two dialogs are part of the contract because the shell still opens them, through
`SvelteIsland`: every prop they take is a plain value or a callback, so no adapter is needed
beyond the island itself. Never add: `organization.module.ts`, `UserSettingsRoute`.

## Data contract

`OrganizationRepository` — `domain/repository/organization.repository.ts` (**default** export):

| Method | Returns |
|---|---|
| `getAll()` | `OrganizationEntity[]` — every org (admin view) |
| `getMine()` | `OrganizationEntity[]` — the current user's orgs |
| `listMembers(organizationId)` | `UserEntity[]` |

`getAll` vs `getMine` is a real distinction — the switcher must use `getMine`. Reach the
repository with `useOrganizationDomain()` **inside a hook only**.

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
[`user`](../user/AGENTS.md), which owns `users` and its index; this module contributes the
`:userId` leaf because the settings page renders an organizations panel. The route composer
merges both halves onto one `users` parent — that is why two modules may declare the same path
segment here without conflicting.

## Rules that bite here

- **`UserSettingsRoute` is a composition seam.** It renders `UserSettingsPage`, imported from
  `features/user`'s public API — one of the two sanctioned page exports in the codebase. Keep
  the wrapper thin; do not copy user logic into it.
- **The camelCase hook filenames (`useOrganizations.ts`, `useCreateOrganization.ts`) violate the
  kebab-case convention.** They predate it. Do not rename them opportunistically — renaming
  moves Lingui message ownership and requires `node scripts/restore-translations.mjs`. New files
  here use `use-{name}.ts`.
- `ORGANIZATION_MEMBERS_QUERY_KEY` is exported so `membership` invalidates the same entry this
  module reads. Never hand-write the key.
- The shell depends on `OrganizationList` / `AddOrganizationDialog`. Changing their props is a
  breaking change for `layout/` — update `ContextSelector` in the same commit.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
