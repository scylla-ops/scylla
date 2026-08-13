import { useQuery, useQueries } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import type { ProjectEntity } from '@/modules/features/project/domain/entities/project.entity.ts';
import type { PipelineMetadata } from '@/modules/features/pipeline/domain/structs/pipeline.struct.ts';

export type PipelineWithProject = PipelineMetadata & { projectName: string };

export const useOrgOverview = () => {
  const { getProjects } = useDependencies().project;
  const { getPipelinesMetadata } = useDependencies().pipeline;
  const organizationId = useContextStore(state => state.organization.id);

  const projectsQuery = useQuery({
    queryKey: ['dashboard', 'projects', organizationId],
    queryFn: async () =>
      (await getProjects.execute(organizationId!, { page: 1, pageSize: 100 })).unwrap(),
    enabled: !!organizationId,
    staleTime: 30_000,
  });

  const projects: ProjectEntity[] = projectsQuery.data?.projects ?? [];

  const pipelineQueries = useQueries({
    queries: projects.map(project => ({
      queryKey: ['dashboard', 'pipelines', project.id],
      queryFn: async () => {
        const result = await getPipelinesMetadata.execute(project.id, { page: 1, pageSize: 100 });
        return { projectId: project.id, projectName: project.name, pipelines: result.unwrap().items };
      },
      staleTime: 30_000,
      enabled: projects.length > 0,
    })),
  });

  const pipelinesLoading = pipelineQueries.some(q => q.isLoading);

  const allPipelines: PipelineWithProject[] = pipelineQueries.flatMap(q => {
    if (!q.data) return [];
    return q.data.pipelines.map(p => ({ ...p, projectName: q.data!.projectName }));
  });

  return {
    projects,
    projectsLoading: projectsQuery.isLoading,
    projectsError: projectsQuery.isError,
    allPipelines,
    pipelinesLoading,
    organizationId,
  };
};
