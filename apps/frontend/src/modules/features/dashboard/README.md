# Dashboard

> [Scylla frontend](../../../../README.md) › `features/` › **dashboard** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

The landing page for an organization, at `/:org/dashboard`. It answers "what is going on right
now" in one screen: which projects exist, which pipelines they hold, and how recent runs went.
Every organization-level redirect in the app ends here.

## A composite view, by design

The dashboard is the clearest example of a module that **owns nothing**. Its `domain` is empty,
it has no `domain/` or `infrastructure/` folder, and it is registered in the DI container purely
so its route and sidebar entry get composed with everyone else's.

All of its data comes from other modules' public APIs:

| Question | Answered by |
|---|---|
| Which organization am I in? | `useContextStore` — [platform/context](../../platform/context/README.md) |
| Which projects can I see? | `useOrganizationProjects` — [project](../project/README.md) |
| Which pipelines exist? | `useOrganizationPipelines` — [pipeline](../pipeline/README.md) |
| How did recent runs go? | `useOrganizationJobs` — [jobs](../jobs/README.md) |
| What may this user open? | `useAuthorization` — [platform/authz](../../platform/authz/README.md) |

`useOrgOverview` stitches those five together and hands the page a single ready-to-render shape,
including a `ProjectAccess` predicate the UI uses to grey out projects the user cannot enter.

## Why the queries live elsewhere

It would be shorter to call `pipelineRepository.getMetadataByOrganizationId()` from here. It
would also be wrong, and the rule exists because of a real failure: two modules querying the
same resource through different repositories produce two cache keys for one thing, and a
mutation invalidating one leaves the other stale on screen.

So the org-wide reads live in the modules that own the resource — `useOrganizationPipelines` in
`pipeline`, `useOrganizationJobs` in `jobs` — even though the dashboard is currently their only
consumer. The dashboard composes; it does not fetch.

The same reasoning explains the empty `domain`. A module's `domain` is its *data surface*; a
module with no data has no surface, and inventing a `dashboardRepository` to look symmetrical
would only give the next contributor somewhere wrong to put a query.

## What is on the page

- **Run activity** (`RunActivityCard`) — recent job volume and outcomes across the organization.
- **Agent outcomes** (`AgentOutcomesChart`) — success/failure trend over time.
- **Projects and pipelines** — the overview list, with inaccessible projects visibly gated
  rather than hidden, so a user can tell that something exists and they lack access to it.

Access is gated on `READ_ORGANIZATION` — the same permission as the projects list, because both
are reads of the organization and this page is the redirect target for anyone landing on an org
without a deeper destination.

## Related modules

- [project](../project/README.md), [pipeline](../pipeline/README.md),
  [jobs](../jobs/README.md) — the three data sources.
- [agents](../agents/README.md) — supplies the `NoAgentsBanner` empty state shown here.
- [layout](../../layout/README.md) — the shell this page renders inside.
