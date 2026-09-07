# `features/user` — agent guide

User accounts: the system-wide directory and a user's own settings.

**Layer** `features/` · **id** `user` · **DI key** `userRepository`

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

## Public API — `index.ts`

```typescript
type UserEntity
useUser, useUsers
UserSettingsPage                      ← the documented page exception
```

`UserSettingsPage` is exported because [`organization`](../organization/AGENTS.md) composes it
behind its own `users/:userId` route, to render the organizations panel. That consumer is
lazily loaded, which is what makes the exception safe. Do not add a second page export.

Never add: `user.module.ts`, `use-user-domain.ts`.

## Data contract

`UserRepository` — `domain/repository/user.repository.ts`:

| Method | Returns |
|---|---|
| `getAll()` | `UserList` (= `PaginatedList<UserEntity>`, from `@shared`) |
| `getById(id)` | `UserEntity` |
| `create(username, password)` | `UserEntity` |
| `update(userId, username?)` | `UserEntity` |
| `delete(userId)` | `void` |

`update` takes **no password** — password changes are not exposed here. Reach the repository
with `useUserDomain()` **inside a hook only**.

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
  hooks/use-user-domain.ts           DI accessor (private)
  hooks/use-user.ts, use-users.ts, use-create-user.ts,
  hooks/use-update-user.ts, use-delete-user.ts
  ui/admin/UserAdmin.page.tsx, AddUserDialog.tsx
  ui/admin/user-table/UserTable.tsx, UserColumns.tsx
  ui/settings/UserSettings.page.tsx, UserInformation.tsx
```

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `organization` | `users` (index) | `LIST_USERS` | `UserAdminPage` |

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

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles` — all clean.
New strings: `pnpm extract && pnpm compile`.
