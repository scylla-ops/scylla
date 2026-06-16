import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import type { PipelineStep } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';

interface EditPipelineParams {
  id: string;
  nodes: PipelineStep[];
  name?: string;
}

export const useUpdatePipeline = () => {
  const { updatePipeline } = useDependencies().pipeline;
  const queryClient = useQueryClient();
  const { goToProject } = useScyllaNavigate();

  const project = useContextStore(state => state.project);
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async ({ id, nodes, name }: EditPipelineParams) =>
      (await updatePipeline.execute(id, nodes, name)).unwrap(),
    onSuccess: data => {
      void queryClient.invalidateQueries({
        queryKey: ['pipelines', data.projectId],
      });
      void queryClient.invalidateQueries({ queryKey: ['pipeline', data.id] });
      toast.success(i18n._(ToastMessages.PIPELINE_UPDATE));
      if (project.name && project.id) goToProject({ id: project.id, name: project.name });
    },
  });
};
