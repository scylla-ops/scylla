import { getQueryClient, mutationOptions, queryOptions } from '@platform/query';
import { getModuleDomain } from '@platform/di';
import { Permission, authorizationReady, can } from '@platform/authz';
import { i18n } from '@lingui/core';
import { toast } from '@shared/presentation/utils/toast.ts';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import { DEFAULT_PAGE_SIZE, type PaginationParams } from '@shared/domain/structs/pagination.struct.ts';
import type { ProjectEntity } from '../domain/entities/project.entity.ts';
import type { ProjectModule } from '../project.module.ts';

/**
 * Everything the project module reads and writes.
 *
 * Replaces the seven hooks, and keeps what mattered about them: one query-key
 * factory shared by the paginated list, the dashboard overview and the
 * grant-label lookup, so the same organization asked for the same page is the
 * same cache entry whoever asks.
 */
const repository = () =>
  getModuleDomain<typeof ProjectModule.domain>('project').projectRepository;

/** The project list of one organization, for one page. */
export const PROJECTS_QUERY_KEY = (
  organizationId: string | null,
  pagination?: PaginationParams,
) => ['projects', organizationId, pagination] as const;

/** Prefix matching every page of every organization — for broad invalidation. */
export const PROJECTS_QUERY_ROOT = ['projects'] as const;

/**
 * One request covers an organization's whole project list for the lookups that
 * need names rather than a page (dashboard overview, grant target labels).
 */
export const PROJECTS_LOOKUP_PAGE: PaginationParams = { page: 1, pageSize: 100 };

export const PROJECT_MEMBERS_QUERY_KEY = (projectId: string) =>
  ['projects', projectId, 'members'] as const;

const FIRST_PAGE: PaginationParams = { page: 1, pageSize: DEFAULT_PAGE_SIZE };

/**
 * Whether this organization's projects may be listed at all.
 *
 * The backend checks `ListProjectsByOrganization` on the organization, and
 * these queries are consumed from `roles` and `core` — outside this module's
 * route guard — so they ask for themselves. An empty list is therefore not
 * proof of "no projects"; a caller that must tell the two apart calls this.
 */
export const canListProjects = (organizationId: string | null): boolean =>
  authorizationReady() &&
  !!organizationId &&
  can(Permission.LIST_PROJECTS_BY_ORGANIZATION, { organizationId });

export const projectQueries = {
  /** One organization's projects, paginated. */
  byOrganization: (organizationId: string | null, pagination: PaginationParams = FIRST_PAGE) =>
    queryOptions({
      queryKey: PROJECTS_QUERY_KEY(organizationId, pagination),
      queryFn: async () =>
        (await repository().getByOrganizationId(organizationId!, pagination)).unwrap(),
      enabled: canListProjects(organizationId),
    }),

  /**
   * Every project of one organization, unpaginated from the caller's point of
   * view — for the pages that need the whole list rather than a page of it.
   *
   * The backend already filters to what the caller may read, so unlike
   * {@link projectQueries.byOrganization} there is no client-side gate: the
   * dashboard calls this for the organization it is already on.
   */
  lookup: (organizationId: string | null) =>
    queryOptions({
      queryKey: PROJECTS_QUERY_KEY(organizationId, PROJECTS_LOOKUP_PAGE),
      queryFn: async () =>
        (await repository().getByOrganizationId(organizationId!, PROJECTS_LOOKUP_PAGE)).unwrap(),
      enabled: !!organizationId,
      staleTime: 30_000,
    }),

  /**
   * Who holds a role scoped to the project.
   *
   * Derived from grants on the backend, so a grant mutation changes it. Those
   * live in `useScopedGrants`, which cannot reach this key without coupling the
   * two features — callers invalidate it with
   * {@link invalidateProjectMembers}.
   */
  members: (projectId: string | null, options: { enabled?: boolean } = {}) =>
    queryOptions({
      queryKey: PROJECT_MEMBERS_QUERY_KEY(projectId ?? ''),
      queryFn: async () => (await repository().listMembers(projectId!)).unwrap(),
      enabled: (options.enabled ?? true) && !!projectId,
    }),
};

export interface ProjectLookupEntry {
  name: string;
  organizationId: string;
}

/**
 * A projectId → {name, organization} lookup across several organizations.
 *
 * A project id on its own says nothing about which organization owns it, so
 * anything resolving ids to names has to fan out. That fan-out is a project
 * concern, so it lives here rather than being rebuilt against the repository by
 * every caller — and it shares {@link PROJECTS_QUERY_KEY} with the paginated
 * list, so an organization already loaded is served from cache.
 *
 * Filtered by permission *before* going out: asking for all of them would mean
 * one denial per organization the caller cannot read.
 *
 * Returns what `useQueries` / `createQueries` take, rather than calling either,
 * so both bindings can run it.
 */
export const projectLookupQueries = (organizationIds: string[], enabled = true) => {
  const readableIds =
    enabled && authorizationReady()
      ? organizationIds.filter(organizationId =>
          can(Permission.LIST_PROJECTS_BY_ORGANIZATION, { organizationId }),
        )
      : [];

  return {
    queries: readableIds.map(organizationId => projectQueries.lookup(organizationId)),
    // Folded here rather than by the caller so TanStack Query can memoize the
    // map on the underlying results instead of rebuilding it every render.
    combine: (results: { data?: { projects: ProjectEntity[] } }[]) => {
      const byProjectId = new Map<string, ProjectLookupEntry>();
      results.forEach((result, index) => {
        // Indexes line up with `readableIds`, which is what was queried.
        const organizationId = readableIds[index];
        for (const project of result.data?.projects ?? []) {
          byProjectId.set(project.id, { name: project.name, organizationId });
        }
      });
      return byProjectId;
    },
  };
};

export const invalidateProjectMembers = (projectId: string | null): void => {
  if (!projectId) return;
  void getQueryClient().invalidateQueries({
    queryKey: PROJECT_MEMBERS_QUERY_KEY(projectId),
    exact: true,
  });
};

const invalidateProjects = () =>
  getQueryClient().invalidateQueries({ queryKey: PROJECTS_QUERY_ROOT });

export const projectMutations = {
  create: () =>
    mutationOptions({
      mutationFn: async ({
        name,
        organizationId,
        description,
      }: {
        name: string;
        organizationId: string;
        description?: string;
      }) => (await repository().create(name, organizationId, description)).unwrap(),
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.PROJECT_CREATE));
        return invalidateProjects();
      },
    }),

  update: () =>
    mutationOptions({
      mutationFn: async ({
        projectId,
        name,
        description,
      }: {
        projectId: string;
        name?: string;
        description?: string;
      }) => (await repository().update(projectId, name, description)).unwrap(),
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.PROJECT_UPDATE));
        return invalidateProjects();
      },
    }),

  remove: () =>
    mutationOptions({
      mutationFn: async (projectId: string) => (await repository().delete(projectId)).unwrap(),
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.PROJECT_DELETE));
        return invalidateProjects();
      },
    }),
};
