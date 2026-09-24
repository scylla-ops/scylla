# `features/project` — agent guide

Projects: the unit that owns pipelines, secrets and its own member list.

**Layer** `features/` · **id** `project` · **DI key** `projectRepository`

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

**Presentation is Svelte** (Phase 2 of `refacto_svelte.md`). Domain and infrastructure are
unchanged. There is no `use-<feature>-domain.ts` and no hooks: reads and writes are declared as
options objects in `presentation/*.queries.ts`, which a component or another feature runs with
`createQuery`.

## Public API — `index.ts`

```typescript
type ProjectEntity, ProjectMember, ProjectLookupEntry
projectQueries        byOrganization (paginated) · lookup (org-wide) · members
projectLookupQueries  the batched fan-out: { queries, combine } for useQueries/createQueries
projectMutations      create · update · remove
canListProjects, invalidateProjectMembers
PROJECTS_QUERY_KEY, PROJECTS_QUERY_ROOT, PROJECTS_LOOKUP_PAGE, PROJECT_MEMBERS_QUERY_KEY
```

`core`, `roles`, `membership` and `dashboard` all run these with `createQuery` /
`createQueries`, which keeps the three ways of reading an organization's projects on one cache
entry.

Never add: `project.module.ts`, pages.

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
  project.queries.ts                 every read and write, plus the key factories
  ui/Project.page.svelte             picks the organization, or says there is none
  ui/ProjectList.svelte              keyed on the organization — that is the page reset
  ui/ProjectHeader.svelte, ProjectCard.svelte
  ui/AddProjectDialog.svelte, EditProjectDialog.svelte
  ui/project.messages.ts
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

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
