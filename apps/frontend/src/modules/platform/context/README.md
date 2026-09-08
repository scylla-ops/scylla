# platform / context

> [Scylla frontend](../../../../README.md) › `platform/` › **context** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

Scylla's UI is scoped: you are always inside an organization, often inside a project, sometimes
looking at a pipeline. This capability holds *which ones*, and builds navigation from that.

Two exports, imported as `@platform/context`: `useContextStore` and `useScyllaNavigate`.

## Identifiers only, on purpose

The store holds an `{ id, name }` pair for each of the three scopes. Nothing more — no
`OrganizationEntity`, no `ProjectEntity`, no members, no settings.

That restriction is what makes the module possible at all. Context is genuinely *business*
information, which is why it lives in `platform/` rather than `shared/`. But `platform/` may
never import a feature, and the moment this store held a `ProjectEntity` it would have to import
[features/project](../../features/project/README.md) — inverting the layering and putting a
cycle between the two.

Keeping it to ids and names means it never needs to know what a project *is*. A component that
needs more reads the id from here and calls the owning feature's hook with it. The store is a
pointer, not a cache.

The same reasoning explains why it is one of only two global stores in the app (the other being
`useSelectionStore` in [shared](../../shared/README.md)). Everything else is either server state
in TanStack Query or local `useState`.

## Switching organizations clears the rest

`setOrganization` resets `project` and `pipeline` to null.

A project id only means something inside its organization. Carrying one across a switch would
build a URL pointing at another tenant's resource, or fire a query for a project the user cannot
see. Clearing is the correct behaviour, and it is worth knowing about before writing code that
assumes `project.id` survives an org change.

The store persists to `localStorage` under `scylla-context`, so a reload keeps you where you
were rather than dumping you at the organization picker.

## The URL is the source of truth

Context is *derived from the route*, not the other way round. The shell's
`OrganizationSync.wrapper.tsx` reads route params and writes them into the store;
`ContextCleaner.wrapper.tsx` clears context when you leave a scope. See
[core](../../core/README.md).

This direction is deliberate. If both the URL and the store could drive each other, you would
need effects on both sides watching for changes — the mirror-state cascade the codebase's React
rules exist to prevent. One writer, one direction, and a pasted URL always wins.

## Navigating

`useScyllaNavigate()` is how you move between scoped screens. It knows the current context, so
callers ask for a destination rather than assembling
`/${orgSlug}/projects/${projectId}/pipelines/${id}/jobs` by hand.

Beyond convenience, it is a single point of change: when a route's shape changes, one file is
updated instead of every template literal scattered across fourteen modules.

## Structure

Two files and a barrel, with no `domain/` or `presentation/` folders. That is intentional
minimalism — the layered structure is mandatory for features that model something; a
two-file capability does not need scaffolding to look like one.

## Related modules

- [core](../../core/README.md) — the wrappers that sync the store from the URL.
- [layout](../../layout/README.md) — the context selector that lets a user switch scope.
- [platform/routing](../routing/README.md) — the mount points (`organization`, `project`) these
  ids correspond to.
