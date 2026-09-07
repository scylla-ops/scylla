# `features/organization` — agent guide

Organizations: the top-level tenant, its members, and the switcher in the shell.

**Layer** `features/` · **id** `organization` · **DI key** `organizationRepository`

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

## Public API — `index.ts`

```typescript
type OrganizationEntity
useOrganizations, useCreateOrganization
useOrganizationMembers, ORGANIZATION_MEMBERS_QUERY_KEY
createOrganizationItems
OrganizationList, AddOrganizationDialog     ← consumed by layout's context selector
```

`OrganizationList` and `AddOrganizationDialog` are part of the contract because the shell builds
the organization switcher from them. Never add: `organization.module.ts`,
`use-organization-domain.ts`, `UserSettingsRoute`.

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
  hooks/use-organization-domain.ts   DI accessor (private)
  hooks/useOrganizations.ts          ⚠ camelCase filename — see below
  hooks/useCreateOrganization.ts     ⚠ camelCase filename
  hooks/use-update-organization.ts, use-delete-organization.ts,
  hooks/use-organization-members.ts
  ui/OrganizationList.tsx, AddOrganizationDialog.tsx, EditOrganizationDialog.tsx
  ui/UserSettingsRoute.tsx           composes user's UserSettingsPage
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

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles` — all clean.
New strings: `pnpm extract && pnpm compile`.
