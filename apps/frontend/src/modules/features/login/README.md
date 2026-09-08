# Login

> [Scylla frontend](../../../../README.md) › `features/` › **login** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

The sign-in screen, and nothing else. This is the smallest feature in the codebase: one route,
one repository method, one hook, two components.

## The session, end to end

Signing in touches three modules, and it helps to see the whole path at once:

1. `LoginForm` collects a username and password and calls `useLogin`.
2. `useLogin` runs the mutation against `LoginRepository.login()`.
3. The gRPC data source calls the backend and, on success, writes `token` and `userId` into
   `localStorage`.
4. Every later request picks the token up: the transport in
   [platform/grpc](../../platform/grpc/README.md) reads `localStorage.token` and attaches
   `Authorization: Bearer …` to each call.
5. `AuthGuard` in [core](../../core/README.md) reads the same key to decide whether a protected
   route renders or redirects back to `/login`.

Notice that the repository returns `ScyllaResult<void>`. The token deliberately never travels
back up through the domain — the data source persists it as a side effect, and everything
downstream reads it from one place. The upside is that no component ever holds a credential; the
cost is that the `localStorage` key is a contract shared by three modules, so changing it means
changing all three together.

Permissions are **not** loaded here. A freshly signed-in user has an empty permission store
until [roles](../roles/README.md)'s `usePermissionSync` fills it — that call needs a repository
and a session, so it happens inside the authenticated shell rather than at sign-in.

## Why it is a module at all

A one-method feature could arguably live in `core/`. It stays a module because the composition
root should not know how authentication works — it should only know that *some* module claims
the `/login` route. `LoginModule` declares `mount: 'public'`, which grafts it outside the auth
guard, and the router is derived from that. If sign-in ever grows an SSO flow, a password reset
or a second factor, it grows here without touching the shell.

## Structure

There is no `entities/` or `structs/` folder, and no mapper. There is nothing to model: the
request is two strings and the response is a token that never becomes a domain object. This is
the "minimalism inside the layers" rule in practice — the four-layer split is mandatory, but an
empty folder to look symmetrical is not.

## Related modules

- [platform/grpc](../../platform/grpc/README.md) — reads the token on every request.
- [core](../../core/README.md) — `AuthGuard` and the public/protected route split.
- [roles](../roles/README.md) — loads the user's effective permissions after sign-in.
- [user](../user/README.md) — the account behind the credentials.
