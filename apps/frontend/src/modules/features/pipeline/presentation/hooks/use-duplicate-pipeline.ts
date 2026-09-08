import { usePipelineDomain } from '@/modules/features/pipeline/presentation/hooks/use-pipeline-domain.ts';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useContextStore } from '@platform/context';
import { useScyllaNavigate } from '@platform/context';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useDuplicatePipeline = () => {
  const { pipelineRepository } = usePipelineDomain();
  const queryClient = useQueryClient();
  const currentProject = useContextStore(state => state.project);
  const { goToProject } = useScyllaNavigate();
  const { i18n, t } = useLingui();

  return useMutation({
    mutationFn: async (pipelineId: string) => {
      const pipelineResult = await pipelineRepository.getById(pipelineId);
      const pipeline = pipelineResult.unwrap();

      const createResult = await pipelineRepository.create({
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
        goToProject(currentProject.id, currentProject.name);
      }
    },
  });
};
