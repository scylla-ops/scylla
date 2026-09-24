# `features/user` — agent guide

User accounts: the system-wide directory and a user's own settings.

**Layer** `features/` · **id** `user` · **DI key** `userRepository`

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
type UserEntity
userQueries, userMutations, canListUsers, USERS_QUERY_KEY, USER_QUERY_KEY
loadUserSettingsPage                  ← the documented page exception, as a loader
```

`roles`, `membership` and `layout` run `userQueries` with `createQuery`, on the same cache entry
as this module's pages.

`loadUserSettingsPage` is exported because [`organization`](../organization/AGENTS.md) composes it
behind its own `users/:userId` route, to render the organizations panel. That consumer is
lazily loaded, which is what makes the exception safe. Do not add a second page export. The
panel is a **snippet** prop now, where it used to be a `ReactNode`.

Never add: `user.module.ts`.

## Data contract

`UserRepository` — `domain/repository/user.repository.ts`:

| Method | Returns |
|---|---|
| `getAll()` | `UserList` (= `PaginatedList<UserEntity>`, from `@shared`) |
| `getById(id)` | `UserEntity` |
| `create(username, password)` | `UserEntity` |
| `update(userId, username?)` | `UserEntity` |
| `delete(userId)` | `void` |

`update` takes **no password** — password changes are not exposed here. The repository is
reached from `presentation/user.queries.ts` and nowhere else.

## Layout

```
user.module.ts                       route + nav + DI wiring (private; registry only)
index.ts                             public API
domain/
  entities/user.entity.ts            UserEntity
  structs/user.struct.ts             UserList = PaginatedList<UserEntity>
  repository/user.repository.ts
infrastructure/
  repository/data-sources/user-remote.data-source.ts     interface
  data/remote/user-remote.data-source.impl.ts            impl (the `.impl.ts` convention)
  repository/mappers/grpc-user.mapper.ts
  repository/default-user.repository.ts
presentation/
  user.queries.ts                    the two reads, the three writes, `canListUsers`
  ui/user.messages.ts                every string the screens show
  ui/admin/UserAdmin.page.svelte, AddUserDialog.svelte
  ui/admin/user-table/UserTable.svelte, user-columns.ts
  ui/settings/UserSettings.page.svelte, UserInformation.svelte
```

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `organization` | `users` (index) | `LIST_USERS` | `UserAdmin.page.svelte` |

Sidebar: section **`system`** (not `organization`), order `10`, icon `UsersIcon`.

**The `users` parent is shared with `organization`**, which declares the `:userId` leaf
(`UserSettingsRoute`) because the settings screen shows an organizations panel. The route
composer merges both halves onto one parent — that is why two modules declare the same segment
without conflicting. If you add a route under `users`, check the other module first.

## Rules that bite here

- **`user` is system-wide, not org-scoped.** It is the directory of every account in the
  installation — hence the `system` sidebar section. Do not filter it by the current
  organization; "who is in this org" is [`membership`](../membership/AGENTS.md)'s question.
- **`UserSettingsPage` is a slot filled by another module.** Changing its props breaks
  `organization`'s `UserSettingsRoute`; update both in the same commit.
- Passwords are write-only: `create` takes one, `update` does not, and nothing reads one back.
  Never log, store or display one.
- `getAll()` is paginated (`UserList`). Use `usePagination()` + `DataTable`; do not render an
  unpaginated directory.
- Users are principals in the authz model — deleting one orphans their grants. Confirm through
  `ConfirmOperationAlertDialog` (`@shared`).

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
