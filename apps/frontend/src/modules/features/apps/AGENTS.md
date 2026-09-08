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
useApps, useApp, useAppSecrets
```

Nothing consumes this barrel yet — the types are exposed because they are the stable part and
cost nothing at runtime. Never add: `apps.module.ts`, `use-apps-domain.ts`, pages.

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

All wrapped in `ScyllaResult`. Reach it with `useAppsDomain()` **inside a hook only**.

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
  hooks/use-apps-domain.ts           DI accessor (private)
  hooks/use-apps.ts                  useApps / useApp / useAppSecrets
  ui/Apps.page.tsx, AppDetails.page.tsx
  ui/components/AppCard.tsx, AppSecretsCard.tsx
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
  are separate cache entries.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
