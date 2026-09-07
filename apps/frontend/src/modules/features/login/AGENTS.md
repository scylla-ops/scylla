# `features/login` — agent guide

Sign-in. Owns the credentials exchange and nothing else.

**Layer** `features/` · **id** `login` · **DI key** `loginRepository`

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

## Public API — `index.ts`

```typescript
useLogin
```

That is the whole surface, and it should stay that way. Never add: `login.module.ts`,
`use-login-domain.ts`, `LoginPage`.

## Data contract

`LoginRepository` — `domain/repository/login.repository.ts`:

| Method | Returns |
|---|---|
| `login(username, password)` | `ScyllaResult<void>` |

**`void` is not an oversight.** The session token never travels back through the domain: the
gRPC data source writes `token` and `userId` into `localStorage` as a side effect of a
successful call. Do not "improve" this by returning the token — see the rule below before
changing anything here.

## Layout

```
login.module.ts                      route + DI wiring (private; registry only)
index.ts                             public API — useLogin only
domain/repository/login.repository.ts
infrastructure/
  repository/data-sources/login-remote.data-source.ts   interface
  data/remote/grpc-login-remote.data-source.ts          impl — writes localStorage
  repository/default-login.repository.ts
presentation/
  hooks/use-login-domain.ts          DI accessor (private)
  hooks/use-login.ts                 the mutation
  ui/Login.page.tsx, LoginForm.tsx
```

No entities, no structs, no mappers — there is nothing to model.

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `public` | `/login` | none | `LoginPage` |

`mount: 'public'` puts it **outside** `AuthGuard`. It must stay there: mounting it anywhere else
makes signing in require being signed in.

No nav entry — the sidebar only renders inside the authenticated shell.

## Rules that bite here

- **Session storage is `localStorage`, keys `token` and `userId`.** Three places touch it and
  they must agree: this data source writes them, `platform/grpc`'s transport reads `token` for
  the `Authorization: Bearer` header, and `core/.../Auth.guard.tsx` reads it to decide whether
  to redirect. Changing the key or the mechanism means changing all three in the same commit.
- No permission gate anywhere in this module — the user has none yet. Permissions are loaded
  *after* sign-in by `usePermissionSync` in [`roles`](../roles/AGENTS.md).
- After a successful login the app must land somewhere the user can actually reach; that
  redirect is the shell's job, not this module's.
- Never log, toast or store the password. Errors surface as a generic failure — do not leak
  whether the username exists.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles` — all clean.
New strings: `pnpm extract && pnpm compile`.
