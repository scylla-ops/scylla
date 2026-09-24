# platform / routing

> [Scylla frontend](../../../../README.md) › `platform/` › **routing** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

The contract a module uses to declare its pages — their paths, permissions, breadcrumbs and
sidebar links — and the router that the shell builds from all the declarations.

Imported as `@platform/routing`.

## The problem it solves

The app used to keep three hand-maintained lists: routes in the router file, links in the
sidebar, and permission checks wrapped around pages. They described the same thing three times,
and they drifted. A link appeared for a page that denied you; a page was reachable that no link
pointed at; a permission was tightened in one list and not the others.

Now each module declares its pages once:

```typescript
export const SecretModule = {
  id: 'secret',
  domain: { secretRepository },
  routes: {
    project: [
      {
        path: 'secrets',
        permission: Permission.LIST_SECRETS,
        breadcrumb: () => ({ label: msg`Secrets` }),
        page: () => import('./presentation/ui/Secret.page.svelte'),
      },
    ],
  },
} satisfies ScyllaModule;
```

The router, the sidebar and the breadcrumbs come from this declaration. A sidebar link is part
of the route (`nav`), and it takes the URL and the permission of the route: a link cannot show
for a page that will deny you.

## Why this lives below the features

Three parts of the app need this contract, and none of them may depend on the others: features
declare routes, [core](../../core/README.md) builds the router, and
[layout](../../layout/README.md) renders the sidebar and the breadcrumbs. `platform/` is below
all three, so they share this vocabulary without a cycle.

## Mounts

A module does not restate the nesting of the app. It puts its routes under a **mount**, and the
shell gives the mount its path, its layout and its wrappers:

| Mount          | Path                                        |
| -------------- | ------------------------------------------- |
| `public`       | `/` — outside the auth guard, e.g. `/login` |
| `app`          | `/` — inside the shell, the landing page    |
| `organization` | `/:organizationSlug`                        |
| `project`      | `/:organizationSlug/projects/:projectId`    |

A module that declares a `project` route gets the auth guard, the layout, the organization
sync, the project clean-up and the "Project" crumb, and never mentions them.

## A tree to write, a flat table to match

A module writes a tree, with `children`, because a tree is easy to read. The router does not
use the tree. At startup, `compileRoutes` flattens all the trees into one table of full paths,
sorted so that the first match is the most specific. Matching a URL is then a loop over a list.

The flat table is also what lets two modules share a path without an import. The crumbs of a
page are the crumbs of each path that its path starts with. The `pipeline` module declares the
"Jobs" crumb on `pipelines/:pipelineId/jobs`; the `jobs` module declares one job on
`pipelines/:pipelineId/jobs/:jobId`; the job page shows both crumbs. Neither module knows the
other.

Each field of a path is declared once. Two modules that set the same field on one path make the
compilation fail at startup and in the tests. There is no silent override.

## Two behaviours worth knowing

**Pages are lazy.** Each `page` is a dynamic import, so each page has its own chunk. The
`permission` and the `breadcrumb` are static data: the guard and the breadcrumbs read them
_without_ loading the chunk. A page that you cannot open is never downloaded.

**A permission guards one page.** A child does not inherit the permission of its parent. What
a page requires is written on the page, where a reader looks for it.

## Why there is no router library

The app first used `sv-router`. The router used only its path matching and its history: its
nested layouts could not give route parameters to the wrappers, its page swap could not keep
the old page for the exit animation, and its merge of route metadata could not build a
breadcrumb trail. The adapter had become larger than the part of the library it used.

With a flat table, the remaining parts are small: a loop to match, `history.pushState`, and a
click listener for links. They are in `runtime/`, under 200 lines with their documentation, and they have their own tests. The router
uses the History API, not the newer Navigation API, because Firefox ESR and jsdom do not have
the Navigation API.

## Related modules

- [core](../../core/README.md) — the mounts, the shell routes and the module registry.
- [layout](../../layout/README.md) — the sidebar and breadcrumb renderers.
- [platform/authz](../authz/README.md) — the `Permission` a route declares.
- [platform/context](../context/README.md) — the navigator that `createAppRouter` returns.
- [platform/di](../di/README.md) — the `domain` field of the same `ScyllaModule`.
