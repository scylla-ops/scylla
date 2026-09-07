import { usePipelineDomain } from '@/modules/features/pipeline/presentation/hooks/use-pipeline-domain.ts';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useContextStore } from '@platform/context';
import { useScyllaNavigate } from '@platform/context';
import type { PipelineEntity } from '@/modules/features/pipeline/domain/entities/pipeline.entity.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const useCreatePipeline = () => {
  const { pipelineRepository } = usePipelineDomain();
  const currentProject = useContextStore(state => state.project);
  const { goToProject } = useScyllaNavigate();
  const { i18n } = useLingui();

  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (pipeline: Omit<PipelineEntity, 'id'>) =>
      (await pipelineRepository.create(pipeline)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.PIPELINE_CREATE));
      if (currentProject.name && currentProject.id) {
        void queryClient.invalidateQueries({ queryKey: ['pipelines', currentProject.id] });
        goToProject(currentProject.id, currentProject.name);
      }
    },
  });
};
