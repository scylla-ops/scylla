import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useMutation } from '@tanstack/react-query';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';

export const useCreatePipeline = () => {
  const { createPipeline } = useDependencies().pipeline;
  const currentProject = useContextStore(state => state.project);
  const { goToProject } = useScyllaNavigate();

  return useMutation({
    mutationFn: async (script: string) => (await createPipeline.execute(script)).unwrap(),
    onSuccess: () => {
      toast.success('Pipeline created successfully');
      if (currentProject.name && currentProject.id) {
        goToProject({ id: currentProject.id, name: currentProject.name });
      }
    },
    onError: error => toast.error(error.message),
  });
};
