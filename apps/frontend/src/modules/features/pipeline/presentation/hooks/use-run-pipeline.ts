import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { toast } from '@shared/presentation/utils/toast.ts';
import { JOBS_QUERY_KEY } from '@/modules/features/pipeline/presentation/hooks/use-pipeline-jobs.ts';
import { useAgents } from '@/modules/features/agents/presentation/hooks/use-agents.ts';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { slugifyOrgName } from '@shared/utils/slug.ts';
import { useNavigate } from 'react-router-dom';

export const useRunPipeline = () => {
  const { runPipeline } = useDependencies().pipeline;
  const queryClient = useQueryClient();
  const { agents } = useAgents();
  const navigate = useNavigate();
  const orgName = useContextStore(state => state.organization.name);

  return useMutation({
    mutationFn: async (pipelineId: string) => (await runPipeline.execute(pipelineId)).unwrap(),
    onSuccess: (_data, pipelineId) => {
      // The run itself succeeded either way — the job is created and queued.
      // But with no connected agent it won't start, so say it up front
      // instead of letting the user stare at a pending spinner.
      if (!agents.some(a => a.connected)) {
        toast.warning('Job queued — no agent connected', {
          action: {
            label: 'Agents',
            onClick: () => void navigate(`${orgName ? `/${slugifyOrgName(orgName)}` : ''}/agents`),
          },
        });
      } else {
        toast.success(`Pipeline ran`);
      }
      void queryClient.invalidateQueries({ queryKey: JOBS_QUERY_KEY(pipelineId), exact: true });
    },
  });
};
