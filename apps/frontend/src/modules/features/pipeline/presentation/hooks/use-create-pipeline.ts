import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useMutation } from '@tanstack/react-query';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import type { Pipeline } from '@/modules/features/pipeline/domain/models/pipeline.model.ts';

export const useCreatePipeline = () => {
  const { createPipeline } = useDependencies().pipeline;
  const currentProject = useContextStore(state => state.project);
  const { goToProject } = useScyllaNavigate();

  return useMutation({
    mutationFn: async (pipeline: Omit<Pipeline, 'id'>) =>
      (await createPipeline.execute(pipeline)).unwrap(),
    onSuccess: () => {
      toast.success('Pipeline created');
      if (currentProject.name && currentProject.id) {
        goToProject({ id: currentProject.id, name: currentProject.name });
      }
    },
  });
};
