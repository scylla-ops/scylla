# User

> [Scylla frontend](../../../../README.md) › `features/` › **user** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

User accounts — the human principals of the system. This module covers two quite different
screens: the administrator's directory of every account, and a single user's settings.

## System-wide, not organization-scoped

The user directory sits in the sidebar's **System** section, not the organization section, and
that placement is the module's defining fact: it lists every account in the Scylla installation,
regardless of which organization you happen to be viewing.

The question "who is in *this* organization?" is a different one, answered by
[membership](../membership/README.md) from grants. Filtering this directory by the current
organization would quietly conflate the two, and an administrator would lose the only view of
all accounts.

## Two screens

**The directory** (`/:org/users`, behind `LIST_USERS`) is a paginated table with create, rename
and delete. Deleting a user is consequential — they are a principal in the authorization model,
so their grants go with them — and goes through a confirmation dialog.

**Settings** (`UserSettingsPage`) shows one user's information plus the organizations they
belong to. It is not routed by this module. Instead it is exported from the public API and
mounted by [organization](../organization/README.md) at `users/:userId`, because the
organizations panel is organization data. It is one of only two pages in the codebase exported
from a feature barrel, and the export is safe because its single consumer is lazily loaded.

That means the `users/` route tree is contributed by two modules — the index here, the `:userId`
leaf there — and the route composer merges them onto one parent. Worth remembering before
adding a third route under the same segment.

## Passwords are write-only

`create(username, password)` takes a password. `update(userId, username?)` does not. Nothing
reads one back, and no UI in this module displays or holds a credential beyond the create form's
submission. Password *changes* are simply not exposed by this frontend today.

## Structure

Standard three layers, with the data source implementation using the `.impl.ts` file suffix
(`user-remote.data-source.impl.ts`) rather than the `data/remote/grpc-*` naming used by newer
modules. Both spellings exist in the codebase; match whichever file you are editing rather than
converting one to the other.

`UserList` is `PaginatedList<UserEntity>` — the generic pagination wrapper from
[shared](../../shared/README.md), so the directory table plugs straight into `usePagination()`.

## Related modules

- [membership](../membership/README.md) — users as members of an org or project.
- [roles](../roles/README.md) — users as grant-holding principals.
- [organization](../organization/README.md) — mounts `UserSettingsPage`.
- [login](../login/README.md) — how a user authenticates.
- [apps](../apps/README.md) — the non-human principals.
