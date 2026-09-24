# `features/secret` — agent guide

Project-scoped secrets, injected into pipeline runs.

**Layer** `features/` · **id** `secret` · **DI key** `secretRepository`

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

**Presentation is Svelte** (Phase 2 of `refacto_svelte.md`). Domain and infrastructure are
unchanged. There is no `use-<feature>-domain.ts` and no hooks: reads and writes are declared as
options objects in `presentation/*.queries.ts`, which a component or another feature runs with
`createQuery`.

## Public API — `index.ts`

```typescript
type SecretEntity, CreateSecretInput, CreateSecretValues
secretQueries, secretMutations, SECRETS_QUERY_KEY
```

`pipeline`'s step dialog consumes `secretQueries.byProject` to offer secret names; it runs it
with `createQuery`, on the same cache entry this module's pages use.

Never add: `secret.module.ts`, pages.

## Data contract

`SecretRepository` — `domain/repository/secret.repository.ts`:

| Method | Returns |
|---|---|
| `listByProjectId(projectId)` | `SecretEntity[]` — **metadata only, no values** |
| `create(CreateSecretInput)` | `SecretEntity` |
| `deleteById(secretId)` | `void` |

`CreateSecretInput` lives beside the interface, in domain. The repository is reached from
`presentation/secret.queries.ts` and nowhere else — never from a component.

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
  secret.queries.ts                  the read and both writes, as options objects
  ui/Secret.page.svelte, CreateSecretDialog.svelte
  ui/secret.messages.ts              every string the screen shows
  ui/components/                     SecretHeader, SecretList, SecretHealthOverview,
                                     SecretPagination, secret-columns.ts
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
- **`SecretHealthOverview` and `SecretPagination` are not mounted by any page** — the page
  renders the header and the list only. They were ported as they stood; the figures in the
  first are placeholders, and the second lost the bold on its three numbers, because `<Trans>`
  could wrap them in elements and a `t()` string cannot.
- A column's `cell` is a Svelte snippet, so it lives in `SecretList.svelte`; `secret-columns.ts`
  keeps the part that is data — `accessorKey`, `header`, `size`, `meta.align`.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
