# platform / grpc

> [Scylla frontend](../../../../README.md) › `platform/` › **grpc** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

The frontend talks to Scylla over **gRPC-Web**, using clients generated from the backend's
`.proto` files by [protobuf-ts](https://github.com/timostamm/protobuf-ts). This module owns the
transport those clients run on.

Two exports, imported as `@platform/grpc`: the `ScyllaGrpcTransport` class, and `grpcTransport`
— the single shared instance every feature uses.

## One transport, created once

`grpcTransport` is instantiated at import time in `index.ts`, and it is the only instance in the
application. Every feature's data source receives it in that feature's `<feature>.module.ts`:

```typescript
const dataSource = new GrpcSecretRemoteDataSource(grpcTransport);
const secretRepository = new DefaultSecretRepository(dataSource);
```

That is the *only* place it is referenced. Hooks, components, repositories and domain files
never see it. Data sources declare it as a type, so importing this module for the type alone
costs nothing at runtime.

## Why it is in `platform/` and not `core/`

The composition root would be the textbook home for a shared client. Putting it there was in
fact the original design, and it caused a real problem: a feature that wanted to build its own
data source had to import `core/`, while `core/` imported every feature to register it. Every
feature and the shell were mutually dependent, and the module graph had a cycle through the
middle of it.

Moving the transport one layer down broke that. `platform/` may not import a feature, and a
feature may import `platform/` — so a module can now declare its own dependencies without
touching the app that assembles them. The cycle is gone, and `pnpm depcruise:cycles` keeps it
that way.

## What it actually does

Two things beyond wrapping the protobuf-ts transport:

1. **Base URL** — from `VITE_API_URL`, so the same build points at any backend.
2. **Authentication** — before each call it reads the session token from `localStorage` and, if
   present, adds `Authorization: Bearer <token>` to the request metadata.

That second point makes the token a contract shared by three modules:
[login](../../features/login/README.md) writes it after a successful sign-in, this transport
reads it on every request, and `AuthGuard` in [core](../../core/README.md) reads it to decide
whether a protected route renders. Changing the key means changing all three together.

## Errors stop here, results travel

The transport does not catch or translate failures. Instead, every data source wraps its call in
`ScyllaResult.tryAsync`, so an RPC failure becomes a `ScyllaError` carrying the extracted gRPC
status code, and travels up as a value rather than an exception. Query and mutation hooks call
`.unwrap()` inside `queryFn` / `mutationFn`, which is where TanStack Query takes over. See
[shared](../../shared/README.md).

## Generated code and the mapper boundary

Proto clients are generated into `src/generated/` by `pnpm gen-proto`, which runs automatically
as part of `prebuild` before `dev` and `build`. **Never hand-edit them** — the next generation
wipes the change.

Just as importantly, generated types stay in infrastructure. Each feature's
`Grpc{Entity}Mapper` converts proto messages into domain entities and structs, and that is where
proto vocabulary ends. When a domain type and a proto type share a name, the proto one is
aliased in the mapper; the domain one carries the `Entity` suffix. A proto type appearing in
`domain/` or `presentation/` is a leak.

## Streaming

Most calls are unary. Log tailing is not: `jobs`' `tailLogs` opens a server stream and returns
its handle **synchronously**, wrapped in a `ScyllaResult`. It is the one non-promise method in
the codebase, and consumers are responsible for closing the stream. See
[jobs](../../features/jobs/README.md).

## Related modules

- [features/login](../../features/login/README.md) — writes the token this transport reads.
- [core](../../core/README.md) — the auth guard reading the same key.
- [platform/di](../di/README.md) — how the repositories built on this transport reach the hooks.
- [shared](../../shared/README.md) — `ScyllaResult`, `ScyllaError`.
