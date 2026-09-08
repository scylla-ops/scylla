# Secret

> [Scylla frontend](../../../../README.md) › `features/` › **secret** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

Secrets are the values a pipeline needs but must not contain: API tokens, registry passwords,
signing keys. They belong to a project and are injected into runs at execution time.

The page lives at `/:org/projects/:projectId/secrets`, behind `LIST_SECRETS`. There is no
sidebar entry — secrets are reached from inside a project.

## The shape of the feature

Three operations, and the *absence* of a fourth:

- **List** — names, scope and timestamps for every secret in the project.
- **Create** — name plus value; the value goes out and never comes back.
- **Delete** — remove it.
- **No update.** Changing a secret means deleting it and creating a new one.

That is a backend constraint, not an oversight, and the UI reflects it rather than papering over
it with an edit dialog that would secretly delete-and-recreate.

## Values are write-only

The single rule worth internalising: **the frontend never sees a secret's value after it is
created.**

`listByProjectId` returns metadata only. There is no `getValue` method, because the backend
exposes none — values are decrypted at run time, inside the runner, never in a browser. The
plaintext exists in exactly one place in this codebase: the create form, on its way to the
server.

So it must never be written to a Zustand store, kept in a query cache, echoed in a toast, or
included in an error message. Any code path that could persist it beyond the form submission is
a bug, regardless of how convenient it is.

## What the page shows

Beyond the list, `SecretHealthOverview` gives an at-a-glance summary — how many secrets exist,
how old they are, whether any look stale. It is a nudge toward rotation, built entirely from the
metadata the list already returns, so it costs no extra request.

The list itself is a `DataTable` with columns defined in `secret-columns.tsx`, paginated through
the shared `usePagination` hook, with a `FeatureHeader` carrying the count and the new-secret
button.

## Naming: secret vs credential

The infrastructure file is `grpc-credential-remote.data-source.ts`, because the proto service is
called `Credential`. Everywhere else — the entity, the repository, the hooks, the UI, this
document — the concept is **Secret**.

This is the mapper boundary doing its job: proto vocabulary stops at the edge of
infrastructure, and the domain keeps the name the product uses. If you find "credential" in
`domain/` or `presentation/`, it leaked and should be renamed.

## Related modules

- [pipeline](../pipeline/README.md) — the runs that consume secrets.
- [project](../project/README.md) — the owning scope.
- [apps](../apps/README.md) — machine identities with their own, differently-scoped secrets.
- [shared](../../shared/README.md) — `DataTable`, `usePagination`, `FormDialog`,
  `ConfirmOperationAlertDialog`.
