# `platform/grpc` — agent guide

The single gRPC-Web transport every feature's data sources are wired to.

**Layer** `platform/` · alias `@platform/grpc`

## Import rules

- **MUST NEVER import a feature** (`platform-knows-no-feature`, error).
- Consumers import `@platform/grpc` — the barrel, never a deep path.

## Public API — `index.ts`

```typescript
ScyllaGrpcTransport          the class
grpcTransport                the shared singleton instance
```

## Where it is used

**Only in `<feature>.module.ts` files**, to construct that feature's data source:

```typescript
const dataSource = new GrpcXRemoteDataSource(grpcTransport);
const repository = new DefaultXRepository(dataSource);
```

Never import it in a hook, a component, a repository or a domain file. A data source takes the
transport as a **type** only, so importing it for the type alone costs nothing at runtime.

It lives in `platform/` rather than the composition root precisely so a feature can declare its
own dependencies without importing the app that assembles them — the coupling that previously
made every feature and `core/` mutually dependent.

## Layout

```
index.ts                             re-exports the class + creates `grpcTransport`
scylla-grpc-transport.ts             the transport
```

## What the transport does

- `baseUrl` from `import.meta.env.VITE_API_URL` (empty string if unset).
- **Attaches auth on every call**: reads `localStorage.getItem('token')` and, when present, sets
  `options.meta['Authorization'] = 'Bearer <token>'`.

That `token` key is a three-way contract:

| Who | Does what |
|---|---|
| `features/login`'s gRPC data source | **writes** `token` and `userId` |
| **this module** | **reads** `token` for the header |
| `core/.../Auth.guard.tsx` | **reads** `token` to allow or redirect |

Changing the key or the storage mechanism means changing all three in one commit.

## Rules that bite here

- **One instance, module-scoped.** `grpcTransport` is created at import time in `index.ts`.
  Do not construct a second `ScyllaGrpcTransport` in a feature — the auth interceptor and the
  base URL would then exist in two places.
- Errors are not handled here. Every call returns `ScyllaResult<T>` — wrap with
  `ScyllaResult.tryAsync` in the data source, and let `ScyllaError` extract the gRPC code.
- Generated proto clients live in `src/generated/` and are **never hand-edited** — run
  `pnpm gen-proto`.
- Proto vocabulary stops at infrastructure. Mappers (`Grpc{Entity}Mapper`) convert proto types
  into domain entities/structs; a proto type must never reach `domain/` or `presentation/`.
- Streaming calls (see `jobs`' `tailLogs`) return a handle synchronously rather than a promise.
  Consumers must close them.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
After a `.proto` change: `pnpm gen-proto`.
