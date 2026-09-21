import { usePipelineDomain } from '@/modules/features/pipeline/presentation/hooks/use-pipeline-domain.ts';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import { JOBS_QUERY_KEY } from '@/modules/features/jobs';
import { agentQueries } from '@/modules/features/agents';
import { Permission, useCan } from '@platform/authz';
import { useContextStore } from '@platform/context';
import { slugifyOrgName } from '@shared/utils/slug.ts';
import { useNavigate } from 'react-router-dom';

export const useRunPipeline = () => {
  const { pipelineRepository } = usePipelineDomain();
  const queryClient = useQueryClient();
  // `agents` went Svelte in Phase 3: what was a hook is now an options object
  // react-query takes unchanged, sharing the same cache entry as the Svelte
  // pages. `useCan` is not decoration — it subscribes this hook to the
  // permissions store, which is what makes the query's own `enabled` (a
  // `can(LIST_AGENTS)` inside the factory) re-evaluate once they land.
  const canListAgents = useCan(Permission.LIST_AGENTS);
  const organizationId = useContextStore(state => state.organization.id);
  const { data: agents = [] } = useQuery(agentQueries.byOrganization(organizationId ?? ''));
  const navigate = useNavigate();
  const orgName = useContextStore(state => state.organization.name);
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async (pipelineId: string) => (await pipelineRepository.run(pipelineId)).unwrap(),
    onSuccess: (_data, pipelineId) => {
      // The run itself succeeded either way — the job is created and queued.
      // But with no connected agent it won't start, so say it up front
      // instead of letting the user stare at a pending spinner.
      //
      // Without LIST_AGENTS the agent list is never fetched (it would only be
      // denied), so connectivity is unknown here: point at agents as something
      // to check rather than claim none is connected.
      if (!canListAgents) {
        toast.success(i18n._(ToastMessages.PIPELINE_RUN_CHECK_AGENTS));
      } else if (!agents.some(a => a.connected)) {
        toast.warning(i18n._(ToastMessages.PIPELINE_JOB_QUEUED_WARNING), {
          action: {
            label: 'Agents',
            onClick: () => void navigate(`${orgName ? `/${slugifyOrgName(orgName)}` : ''}/agents`),
          },
        });
      } else {
        toast.success(i18n._(ToastMessages.PIPELINE_RUN));
      }
      void queryClient.invalidateQueries({ queryKey: JOBS_QUERY_KEY(pipelineId) });
    },
  });
};
