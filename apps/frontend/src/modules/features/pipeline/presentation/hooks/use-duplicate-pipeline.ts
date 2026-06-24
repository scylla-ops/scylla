import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';

export const useDuplicatePipeline = () => {
  const { createPipeline, getPipeline } = useDependencies().pipeline;
  const queryClient = useQueryClient();
  const currentProject = useContextStore(state => state.project);
  const { goToProject } = useScyllaNavigate();

  return useMutation({
    mutationFn: async (pipelineId: string) => {
      const pipelineResult = await getPipeline.execute(pipelineId);
      const pipeline = pipelineResult.unwrap();

      const createResult = await createPipeline.execute({
        name: `${pipeline.name} (copy)`,
        projectId: pipeline.projectId,
        nodes: pipeline.nodes,
      });
      createResult.unwrap();
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['pipelines'] });
      toast.success('Pipeline duplicated successfully');
      if (currentProject.name && currentProject.id) {
        goToProject({ id: currentProject.id, name: currentProject.name });
      }
    },
  });
};
