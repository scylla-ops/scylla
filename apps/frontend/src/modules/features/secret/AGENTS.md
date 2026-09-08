# `features/secret` — agent guide

Project-scoped secrets, injected into pipeline runs.

**Layer** `features/` · **id** `secret` · **DI key** `secretRepository`

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

## Public API — `index.ts`

```typescript
type SecretEntity, CreateSecretInput
useSecrets, useCreateSecret, useDeleteSecret
```

Never add: `secret.module.ts`, `use-secret-domain.ts`, pages.

## Data contract

`SecretRepository` — `domain/repository/secret.repository.ts`:

| Method | Returns |
|---|---|
| `listByProjectId(projectId)` | `SecretEntity[]` — **metadata only, no values** |
| `create(CreateSecretInput)` | `SecretEntity` |
| `deleteById(secretId)` | `void` |

`CreateSecretInput` lives beside the interface, in domain. Reach the repository with
`useSecretDomain()` **inside a hook only**.

There is **no `update`**: a secret is replaced by delete + create, and no method returns a
stored value. Both are backend constraints — do not add hooks pretending otherwise.

## Layout

```
secret.module.ts                     route + DI wiring (private; registry only)
index.ts                             public API
domain/
  entities/secret.entity.ts          SecretEntity, CreateSecretInput
  repository/secret.repository.ts
infrastructure/
  data/grpc-credential-remote.data-source.ts             impl (⚠ "credential", not "secret")
  repository/data-sources/secret-remote.data-source.ts   interface
  repository/mappers/grpc-secret.mapper.ts
  repository/default-secret.repository.ts
presentation/
  hooks/use-secret-domain.ts         DI accessor (private)
  hooks/use-secrets.ts               all three hooks in one file
  ui/Secret.page.tsx, CreateSecretDialog.tsx
  ui/components/                     SecretHeader, SecretList, SecretHealthOverview,
                                     SecretPagination, secret-columns.tsx
  ui/components/index.ts             local barrel — internal, not the module's public API
  utils/createSecretItems.ts         ⚠ camelCase filename
```

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `project` | `secrets` | `LIST_SECRETS` | `SecretPage` |

**No nav entry** — reached from inside a project, not the organization sidebar.

## Rules that bite here

- **Never render a secret value.** `listByProjectId` returns metadata (name, scope, timestamps)
  and the backend does not return values at all. If something looks like a value, it isn't —
  check the mapper before displaying it.
- The plaintext exists exactly once: in the create form, on its way out. Do not put it in a
  store, a query cache, a toast, a log or an error message.
- The data source file is named `grpc-credential-remote.data-source.ts` because the proto
  service is `Credential`. Domain-side the concept is **Secret** everywhere. Keep the proto name
  inside infrastructure; never let "credential" leak into domain or presentation.
- `createSecretItems.ts` is camelCase, predating the kebab-case convention. Do not rename
  opportunistically — that moves Lingui message ownership and needs
  `node scripts/restore-translations.mjs`. New files use kebab-case.
- Deletion is destructive and will break running pipelines that depend on the secret. Confirm
  through `ConfirmOperationAlertDialog` (`@shared`).

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
