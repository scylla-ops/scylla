# Layout

> [Scylla frontend](../../../README.md) › `app/` › **layout** ·
> [agent guide](./AGENTS.md) · [architecture](../../../docs/architecture.md)

The application shell — everything that stays on screen while pages come and go: the sidebar,
the top bar, the breadcrumb trail, the organization and project selector, the language switch
and the user menu.

Together with [core](../core/README.md) it forms the `app/` layer, at the top of the dependency
graph. `core` decides *what* is composed; `layout` decides how it *looks*.

## The sidebar is derived, not written

There is no list of links in this module.

`core` calls `navEntriesFor(modules)` and passes the result into `Layout`. Every entry comes
from a module's own `nav` declaration — its title, its icon, its order and, crucially, its
`permission`. That permission is the same one the module's route declares, so the link and the
page it points at cannot disagree: a link is never shown for a page that will refuse you, and
never hidden for one that would let you in.

The practical consequence is that adding a page to the sidebar is a change to that module's
`<feature>.module.ts`. Nothing in `layout/` moves.

Entries are grouped into two sections — **organization** (dashboard, projects, members, agents,
marketplace) and **system** (users, roles) — which reflects the real distinction between
"inside the organization you are viewing" and "across the whole installation".

## Breadcrumbs come from route handles

`ScyllaBreadcrumbs` does not parse the pathname. It reads the `breadcrumb` function each matched
route stored in its `handle`, and renders the resulting crumbs.

A crumb keeps translatable words apart from data: `label` and `detail` are Lingui message
descriptors, `highlight` is the resource's name shown verbatim in every locale. So
`Pipeline · my-deploy · Jobs` translates its first and last parts and leaves the middle alone.

Because handles are static metadata, this works alongside lazily loaded pages — the trail
renders without waiting for the page's chunk.

## The context selector

The organization switcher at the top of the sidebar is where `layout` reaches into a feature.
`ContextSelector` composes `OrganizationList` and `AddOrganizationDialog`, imported from
[organization](../features/organization/README.md)'s public API, and writes the chosen
organization into the [context store](../platform/context/README.md).

That is the reason those two components are exported from a feature barrel at all — they are
part of that module's contract precisely because the shell renders them.

The dependency runs one way only: the shell may import a feature, a feature may never import the
shell. Dependency-cruiser enforces both halves. If a feature ever appears to need something from
`layout/`, the thing it needs is generic and belongs in [shared](../shared/README.md).

## Announcing a release: `whats-new.ts`

Announcements used to be hand-written components — one popup for triggers, one badge glued onto
`PipelineActions`, another flag threaded through `FeatureHeader`. Every announcement cost code in
three modules, and retiring it cost the same again, so the code outlived the news.

Now there is one declaration, [`whats-new.ts`](whats-new.ts), and everything else is derived from
it: the dialog on first launch lists its highlights, and a highlight that names a `navUrl` puts a
"New" pill on that sidebar entry. Announcing a feature is adding an entry; retiring the whole
announcement is emptying the list. Nothing else in the app knows an announcement exists.

Two deliberate choices:

- **TypeScript, not JSON.** The copy has to go through Lingui like every other string, and an
  icon is a component. A JSON file would need a translation table and a name→icon map beside it
  to carry the same information, and neither would be type-checked.
- **The seen flags are keyed by `version`.** One `localStorage` key per announcement, prefixed
  with the release it belongs to, so bumping `version` re-arms every announcement at once and no
  one has to invent fresh ids to make the dialog show again.

The dialog is dismissed as a whole; a nav badge clears on its own when the user opens the page it
points at — it has said what it had to say by then.

It is still not a notification system. It announces what shipped in *this* version, to a user who
was here for the last one.

## Related modules

- [core](../core/README.md) — mounts this shell and supplies the nav entries.
- [platform/routing](../platform/routing/README.md) — `NavEntry` and the breadcrumb contract.
- [platform/context](../platform/context/README.md) — where the selector writes.
- [features/organization](../features/organization/README.md) — supplies the switcher's pieces.
- [shared](../shared/README.md) — the shadcn primitives everything here is built from.
