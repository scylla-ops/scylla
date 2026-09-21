# `features/apps` — agent guide

Machine identities ("apps") and the secrets they authenticate with.

**Layer** `features/` · **id** `apps` · **DI key** `appsRepository`

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

## Public API — `index.ts`

```typescript
type AppEntity, AppSecretEntity
type CreatedApp, CreatedAppSecret
appQueries, appMutations
APPS_QUERY_KEY, APP_QUERY_KEY, APP_SECRETS_QUERY_KEY
```

Nothing consumes this barrel yet — the types are exposed because they are the stable part and
cost nothing at runtime. Never add: `apps.module.ts`, pages.

**This module is Svelte** (Phase 3). There is no `use-apps-domain.ts` and no hooks: the
factories in `presentation/apps.queries.ts` resolve the repository through
`getModuleDomain('apps')` per call, and a component runs them with `createQuery` /
`createMutation` from `@platform/query`.

## Data contract

`AppsRepository` — `domain/repository/apps.repository.ts`:

| Method | Returns |
|---|---|
| `listApps(organizationId)` | `AppEntity[]` |
| `getApp(appId)` | `AppEntity` |
| `createApp(organizationId, name)` | `CreatedApp` (carries the one-time secret) |
| `deleteApp(appId)` | `void` |
| `setAppActive(appId, active)` | `AppEntity` |
| `listAppSecrets(appId)` | `AppSecretEntity[]` |
| `createAppSecret(appId, label)` | `CreatedAppSecret` (one-time value) |
| `revokeAppSecret(secretId)` | `void` |
| `setAppSecretEnabled(secretId, enabled)` | `AppSecretEntity` |

All wrapped in `ScyllaResult`. Reach it from `apps.queries.ts` only — never from a component.

## Layout

```
apps.module.ts                       DI wiring only — no routes, no nav (see below)
index.ts                             public API
domain/
  entities/app.entity.ts             AppEntity, AppSecretEntity
  structs/app.struct.ts              CreatedApp, CreatedAppSecret
  repository/apps.repository.ts      the contract
infrastructure/
  data/apps-remote.data-source.ts    interface + Impl, colocated
  data/grpc-app.mapper.ts            GrpcAppMapper
  repository/default-apps.repository.ts
presentation/
  apps.queries.ts                    every read and write, as query/mutation options
  ui/Apps.page.svelte, AppDetails.page.svelte
  ui/apps.messages.ts                every string — `lingui extract` cannot read `.svelte`
  ui/components/AppCard.svelte, AppSecretsCard.svelte
  utils/create-app-form-items.ts, create-app-secret-form-items.ts
```

## Routes & nav

**None.** `AppsModule` declares `domain` only. The pages exist and work, but no route mounts
them and no sidebar entry points at them — the feature is complete below the UI and not yet
surfaced.

If you are asked to expose it: add `routes` + `nav` to `apps.module.ts` (mount `organization`,
gate on an apps permission such as `LIST_APPS_BY_ORGANIZATION`, `lazy`-import the pages). Do
**not** register a route anywhere else — the module declaration is the only door.

## Rules that bite here

- **One-time secrets.** `CreatedApp` / `CreatedAppSecret` carry a plaintext value the backend
  will never return again. Render it through `SecretRevealDialog` (`@shared`), never persist it
  in a store, and never log it.
- Enable/disable is `setAppActive` / `setAppSecretEnabled`, distinct from delete/revoke. Keep
  them distinct in the UI too — disabling is reversible, revoking is not.
- Invalidate the secrets query after any secret mutation; the app query and the secrets query
  are separate cache entries. `appMutations` already does it — do not re-invalidate at the call
  site.
- **New strings go in `ui/apps.messages.ts`, never inside a `.svelte`.** Extraction does not read
  components, so a message declared there vanishes from the catalogs without failing a gate.
- Both icon-only controls on `AppCard` carry an `sr-only` label (`App actions`, `Delete app`).
  The React original had none, so the only way to reach them in a test was a CSS selector. Keep
  the labels.
- **A bits-ui menu lives in a floating layer**, which floating-ui leaves `visibility: hidden`
  under jsdom: `getByRole('menuitem')` finds nothing. Use `findFloating` from
  `src/test/render.svelte.ts`.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
