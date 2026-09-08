# Project

> [Scylla frontend](../../../../README.md) › `features/` › **project** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

A project is the working unit inside an organization. It owns pipelines, secrets and its own
member list, and it is the second segment of nearly every deep URL:
`/:org/projects/:projectId/…`.

This module owns the project entity itself — the list, the CRUD, and the card UI. What lives
*inside* a project belongs to other modules.

## What it covers

- **The projects list** at `/:org/projects`, paginated.
- **Creating, editing and deleting** a project.
- **Project members** — the raw list of principals holding a project-scoped grant. The member
  *experience* is [membership](../membership/README.md)'s.
- **Reads for other modules** — the dashboard overview and the grant-label lookup.

Access is gated on `READ_ORGANIZATION`, because listing an organization's projects is a read of
the organization.

## Three readers, one cache entry

Three different screens need "the projects of an organization", and all three hooks live here:

| Hook | Used by | Shape |
|---|---|---|
| `useProjects` | this module's list page | paginated |
| `useOrganizationProjects` | [dashboard](../dashboard/README.md) | full overview |
| `useProjectsByOrganizations` | [roles](../roles/README.md) grant labels | batched lookup |

It would be easy to let each consumer build its own query. The rule against it exists because
two modules querying the same resource through different repositories produce two cache keys for
one thing — and a mutation invalidating one leaves the other showing stale data on screen.

So the queries live with the module that owns the resource, and consumers import the hook. If a
fourth reader appears, its hook belongs here too.

`useProjectsByOrganizations` is also the codebase's reference for batching: it resolves projects
across several organizations with `useQueries` rather than a loop of `useQuery`, which is what
keeps the grant list from firing one request per row.

## Where the project's contents live

This module declares exactly **one** route — the projects index. Everything under
`/:org/projects/:projectId/` is declared by the module that owns that content:

- [pipeline](../pipeline/README.md) — the project index and the editor.
- [secret](../secret/README.md) — `secrets`.
- [membership](../membership/README.md) — `members`.
- [triggers](../triggers/README.md) — `pipelines/:pipelineId/triggers`.

They do it by declaring `mount: 'project'`, and the shell grafts them under the project shell —
which is why this module does not need to know they exist, and why adding a project-scoped
screen never means editing a router file.

## A note on `ProjectMember`

`ProjectMember` is a struct, not an entity, and it is worth reading its meaning carefully: it
describes **who holds a grant scoped to this project**, not a stored membership row. Scylla has
no separate member table — membership is derived from grants. This module returns the raw list;
`membership` turns it into direct-vs-inherited roles with actions.

## Structure

Standard three layers. Two mappers, because `listMembers` returns member data rather than
project data. Two hook files (`useProjects.ts`, `useCreateProject.ts`) are still camelCase from
before the kebab-case convention — deliberately not renamed, since renaming a file moves its
Lingui message ownership and drops translations unless the restore script runs.

## Related modules

- [organization](../organization/README.md) — the parent scope.
- [pipeline](../pipeline/README.md), [secret](../secret/README.md),
  [triggers](../triggers/README.md) — what a project contains.
- [membership](../membership/README.md) — the member experience.
- [platform/context](../../platform/context/README.md) — holds the active project.
