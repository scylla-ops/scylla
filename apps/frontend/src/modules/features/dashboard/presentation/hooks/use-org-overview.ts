import { useQuery, useQueries } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import type { ProjectEntity } from '@/modules/features/project/domain/entities/project.entity.ts';
import type { PipelineMetadata } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';
import { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { useAuthorization } from '@/modules/features/permission/presentation/hooks/use-authorization.ts';

export type PipelineWithProject = PipelineMetadata & { projectName: string };

/** Project ids the user may open — the project route needs this same permission. */
export type ProjectAccess = (projectId: string) => boolean;

export const PROJECTS_OVERVIEW_QUERY_KEY = (organizationId: string | null) =>
  ['dashboard', 'projects', organizationId] as const;

export const PIPELINES_OVERVIEW_QUERY_KEY = (projectId: string) =>
  ['dashboard', 'pipelines', projectId] as const;

/**
 * The organization-wide overview behind the dashboard: every project the user
 * can see, and the pipelines of those projects.
 *
 * The project list needs no client-side filter — the backend returns only what
 * the caller may read. The pipeline fan-out does: `ListPipelinesByProject` is
 * enforced per project, so asking about a project the user holds no such grant
 * on is a guaranteed `PERMISSION_DENIED`. That is not harmless here — the app's
 * global `queryCache.onError` toasts every failed query, so an unguarded
 * fan-out means one error toast per inaccessible project on each page visit.
 * Gating `enabled` on `can(...)` keeps those requests from being made at all.
 */
export const useOrgOverview = () => {
  const { getProjects } = useDependencies().project;
  const { getPipelinesMetadata } = useDependencies().pipeline;
  const organizationId = useContextStore(state => state.organization.id);
  const { can, ready } = useAuthorization();

  const projectsQuery = useQuery({
    queryKey: PROJECTS_OVERVIEW_QUERY_KEY(organizationId),
    queryFn: async () =>
      (await getProjects.execute(organizationId!, { page: 1, pageSize: 100 })).unwrap(),
    enabled: !!organizationId,
    staleTime: 30_000,
  });

  const projects: ProjectEntity[] = projectsQuery.data?.projects ?? [];

  // Per project, not for the ambient context: `can` defaults its target to the
  // *currently selected* project, which on an org-level page is either unset or
  // left over from wherever the user was last.
  const canListPipelines = (projectId: string) =>
    can(Permission.LIST_PIPELINES_BY_PROJECT, { projectId });

  const pipelineQueries = useQueries({
    queries: projects.map(project => ({
      queryKey: PIPELINES_OVERVIEW_QUERY_KEY(project.id),
      queryFn: async () => {
        const result = await getPipelinesMetadata.execute(project.id, { page: 1, pageSize: 100 });
        return {
          projectId: project.id,
          projectName: project.name,
          pipelines: result.unwrap().items,
        };
      },
      staleTime: 30_000,
      // Permissions unknown → ask nothing, rather than ask and be denied.
      enabled: ready && canListPipelines(project.id),
    })),
  });

  // Until permissions land every query above is disabled, so "no query is
  // loading" would otherwise read as "no pipelines" and flash an empty state.
  const pipelinesLoading = !ready || pipelineQueries.some(q => q.isLoading);

  const allPipelines: PipelineWithProject[] = pipelineQueries.flatMap(q => {
    const data = q.data;
    if (!data) return [];
    return data.pipelines.map(p => ({ ...p, projectName: data.projectName }));
  });

  return {
    projects,
    projectsLoading: projectsQuery.isLoading,
    projectsError: projectsQuery.isError,
    allPipelines,
    pipelinesLoading,
    organizationId,
    /** Whether opening this project would land on something the user may see. */
    canOpenProject: canListPipelines satisfies ProjectAccess,
  };
};
