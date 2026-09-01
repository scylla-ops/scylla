import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useDuplicatePipeline = () => {
  const { createPipeline, getPipeline } = useDependencies().pipeline;
  const queryClient = useQueryClient();
  const currentProject = useContextStore(state => state.project);
  const { goToProject } = useScyllaNavigate();
  const { i18n, t } = useLingui();

  return useMutation({
    mutationFn: async (pipelineId: string) => {
      const pipelineResult = await getPipeline.execute(pipelineId);
      const pipeline = pipelineResult.unwrap();

      const createResult = await createPipeline.execute({
        name: t`${pipeline.name} (copy)`,
        projectId: pipeline.projectId,
        nodes: pipeline.nodes,
      });
      createResult.unwrap();
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['pipelines'] });
      toast.success(i18n._(ToastMessages.PIPELINE_DUPLICATE));
      if (currentProject.name && currentProject.id) {
        goToProject({ id: currentProject.id, name: currentProject.name });
      }
    },
  });
};
