# `features/project` — agent guide

Projects: the unit that owns pipelines, secrets and its own member list.

**Layer** `features/` · **id** `project` · **DI key** `projectRepository`

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

## Public API — `index.ts`

```typescript
type ProjectEntity, ProjectMember
PROJECTS_QUERY_KEY
useProjects                          paginated list
useOrganizationProjects              org-wide overview
useProjectsByOrganizations, type ProjectLookupEntry   batched lookup (useQueries)
useProjectMembers, PROJECT_MEMBERS_QUERY_KEY
```

Never add: `project.module.ts`, `use-project-domain.ts`, pages.

## Data contract

`ProjectRepository` — `domain/repository/project.repository.ts`:

| Method | Returns |
|---|---|
| `getByOrganizationId(organizationId, pagination?)` | `ProjectList` |
| `listMembers(projectId)` | `ProjectMember[]` — holders of a project-scoped grant |
| `create(name, organizationId, description?)` | `ProjectEntity` |
| `update(projectId, name?, description?)` | `ProjectEntity` |
| `delete(projectId)` | `void` |

Reach it with `useProjectDomain()` **inside a hook only**.

## Layout

```
project.module.ts                    route + nav + DI wiring (private; registry only)
index.ts                             public API
domain/
  entities/project.entity.ts         ProjectEntity
  structs/project.struct.ts          ProjectList
  structs/project-member.struct.ts   ProjectMember
  repository/project.repository.ts
infrastructure/
  data/grpc-project-remote.data-source.ts                 impl
  repository/data-sources/project-remote.data-source.ts   interface
  repository/mappers/grpc-project.mapper.ts
  repository/mappers/grpc-project-member.mapper.ts
  repository/default-project.repository.ts
presentation/
  hooks/projects.query-keys.ts       key factory — import it, never inline a key
  hooks/use-project-domain.ts        DI accessor (private)
  hooks/useProjects.ts               ⚠ camelCase filename — see below
  hooks/useCreateProject.ts          ⚠ camelCase filename
  hooks/use-organization-projects.ts, use-projects-by-organizations.ts,
  hooks/use-project-members.ts, use-update-project.ts, use-delete-project.ts
  ui/ProjectPage.tsx, ProjectHeader.tsx, ProjectCard.tsx,
  ui/AddProjectDialog.tsx, EditProjectDialog.tsx
```

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `projects` | index | `READ_ORGANIZATION` | `ProjectPage` |

Sidebar: section `organization`, order `20`, icon `WorkflowIcon`, same permission.

`READ_ORGANIZATION` is the real gate — listing projects *is* reading the organization. Deeper
project routes (`mount: 'project'`) are declared by the modules that own them
([pipeline](../pipeline/AGENTS.md), [secret](../secret/AGENTS.md),
[membership](../membership/AGENTS.md)), not here.

## Rules that bite here

- **Three ways to read an organization's projects, one cache.** `useProjects` (paginated list),
  `useOrganizationProjects` (dashboard overview) and `useProjectsByOrganizations` (grant-label
  lookup) are all exported from this module precisely so no other module rebuilds a project
  query against `projectRepository`. Add the fourth here too.
- `useProjectsByOrganizations` uses `useQueries` to batch — never loop `useQuery`.
- `ProjectMember` is a **struct**, and it means *holders of a project-scoped grant*, not a
  stored roster. The member UI belongs to [`membership`](../membership/AGENTS.md); this module
  supplies the raw list only.
- **The camelCase hook filenames (`useProjects.ts`, `useCreateProject.ts`) violate the
  kebab-case convention.** They predate it. Do not rename them opportunistically — that moves
  Lingui message ownership and needs `node scripts/restore-translations.mjs`. New files use
  `use-{name}.ts`.
- Lists use `DataTable` + `usePagination()`; row keys are project ids.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles` — all clean.
New strings: `pnpm extract && pnpm compile`.
