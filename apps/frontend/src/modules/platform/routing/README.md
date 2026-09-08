# platform / routing

> [Scylla frontend](../../../../README.md) › `platform/` › **routing** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

The contract a module uses to declare itself — its routes, its sidebar entries, its breadcrumbs —
and the composer that turns every module's declaration into a working router.

Imported as `@platform/routing`.

## The problem it solves

The app used to keep three hand-maintained lists: routes in the router file, links in the
sidebar, and permission checks wrapped around pages. They described the same thing three times,
and they drifted. A link appeared for a page that denied you; a page was reachable that no link
pointed at; a permission was tightened in one list and not the others.

Now there is one declaration per module:

```typescript
export const SecretModule = {
  id: 'secret',
  domain: { secretRepository },
  routes: [{
    mount: 'project', path: 'secrets',
    permission: Permission.LIST_SECRETS,
    breadcrumb: () => ({ label: msg`Secrets` }),
    lazy: async () => ({ Component: (await import('./presentation/ui/Secret.page.tsx')).SecretPage }),
  }],
} satisfies ScyllaModule;
```

and the router, the sidebar and the breadcrumb trail are all *derived* from it. `permission` is
written once and read by both the route guard and the sidebar link, so the two cannot disagree.

## Why this lives below the features

Three parts of the app need this contract, and none of them may depend on the others: features
declare routes, [core](../../core/README.md) composes them into a router, and
[layout](../../layout/README.md) reads the breadcrumb shape to render the trail. Putting
`ScyllaModule` in `platform/` — below all three — is what lets them share a vocabulary without a
cycle.

## Mount points

A module does not restate the app's nesting. It names a **scope**, and the shell grafts the
route there:

| Mount | Grafted under |
|---|---|
| `public` | outside the auth guard — `/login` |
| `organization` | `/:organizationSlug` |
| `projects` | `/:organizationSlug/projects` |
| `project` | `/:organizationSlug/projects/:projectId` |

The shell owns the skeleton: the auth guard, the layout, and the wrappers that sync the active
organization and project. A module that declares `mount: 'project'` gets all of that for free
and never mentions it.

## Two behaviours worth knowing

**Routes are lazy.** Every page is behind `routes.lazy`, which is what keeps them out of the
initial chunk. But the route's `permission` and `breadcrumb` are stored in react-router's static
`handle`, so the guard and the breadcrumbs can read them *without* loading the chunk. A page you
cannot access is never downloaded, and its breadcrumb still renders correctly on the way past.

**Sibling routes on the same segment are merged.** `mergeSharedParents` folds routes that claim
the same path into one. That is how [user](../../features/user/README.md) can own `users` (the
directory) while [organization](../../features/organization/README.md) owns `users/:userId` (the
settings page, because it renders the organizations panel) — each declares its own part, neither
imports the other, and the shared ancestor's breadcrumb applies to both.

`RouteGuard` completes the picture: a single pathless layout route per mount point, reading the
deepest declared permission from the matched handles. It replaced fifteen near-identical
`RequirePermission` wrappers that had to be kept in step with the sidebar by hand.

## Breadcrumbs: words and data

A `Crumb` separates what is translated from what is not:

```typescript
{ label: msg`Pipeline`, highlight: pipelineName, detail: msg`Jobs` }
```

`label` and `detail` go through Lingui; `highlight` is business data and stays verbatim in every
locale. They are message *descriptors* rather than JSX, which is what allows a module to declare
its routes in a plain `.ts` file — and because they are resolved at render time, switching
language updates the trail immediately.

## Related modules

- [core](../../core/README.md) — the shell skeleton and the module registry.
- [layout](../../layout/README.md) — the sidebar and breadcrumb renderers.
- [platform/authz](../authz/README.md) — the `Permission` a route declares.
- [platform/di](../di/README.md) — the `domain` field of the same `ScyllaModule`.
