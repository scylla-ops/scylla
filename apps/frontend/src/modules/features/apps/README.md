# Apps

> [Scylla frontend](../../../../README.md) › `features/` › **apps** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

An **app** is a machine identity: a non-human principal that authenticates to Scylla with a
secret instead of a password. Where [agents](../agents/README.md) are runners that execute
work, apps are API clients — CI systems, scripts, integrations — that call Scylla.

## What it covers

- **The app list** — every machine identity in the organization, active or disabled.
- **App details** — an app's metadata and the secrets issued to it.
- **Secret lifecycle** — issuing a labelled secret, disabling it temporarily, revoking it for
  good.
- **Activation** — an app can be deactivated without being deleted, which cuts off every one of
  its secrets at once.

## Not routed yet

`apps.module.ts` declares its `domain` and nothing else: no `routes`, no `nav`. The repository,
the hooks and both pages are written and working, but nothing mounts them, so the feature is
currently invisible in the running app.

This is deliberate — the module was built ahead of the decision about where apps belong in the
navigation. Surfacing it is a small change confined to `apps.module.ts`: add a route mounted on
`organization`, gate it on the right permission, and add the sidebar entry. Because routes and
nav are *derived* from the module declaration, that one file is the whole job.

## How it is built

**Domain** carries two entities — `AppEntity` (the identity) and `AppSecretEntity` (a credential
belonging to it) — plus the `CreatedApp` / `CreatedAppSecret` structs. Those two are separate
from the entities on purpose: they are the *creation response*, and they hold the plaintext
secret the backend returns exactly once.

**Infrastructure** implements `AppsRepository` over gRPC-Web, with the data source interface and
its implementation colocated in a single file and `GrpcAppMapper` doing proto → domain.

**Presentation** offers `useApps`, `useApp` and `useAppSecrets`, and two pages composed from
`AppCard` and `AppSecretsCard`. Both creation dialogs are declarative `ScyllaForm`s built from
the `create-app-form-items` helpers.

## The one-time secret rule

When you create an app or issue a secret, the response contains a plaintext value that will
never be retrievable again. The UI shows it once, in a reveal dialog, with a copy button — and
that is the end of it. Never write it to a Zustand store, never put it in a query cache that
outlives the dialog, never log it. Losing it means issuing a new secret; leaking it means
revoking one.

## Related modules

- [agents](../agents/README.md) — the other machine identity: runners rather than callers.
- [secret](../secret/README.md) — project-scoped secrets injected into pipeline runs. Different
  thing, similar reveal-once handling.
- [roles](../roles/README.md) — an app is a principal that can hold grants.
