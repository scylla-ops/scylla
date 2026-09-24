import { i18n } from '@lingui/core';
import { SvelteSet } from 'svelte/reactivity';
import { Permission, can } from '@platform/authz';
import { scyllaNavigate, contextStore } from '@platform/context';
import { createMutation, createQuery } from '@platform/query';
import { agentQueries } from '@/modules/features/agents';
import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import { pipelineMutations } from './pipeline.queries.ts';

/**
 * Runs a pipeline and warns when no agent is connected (the job would wait).
 * Without `LIST_AGENTS`, connectivity is unknown: the message says to check the agents.
 */
export const createRunPipeline = () => {
  const context = toRune(contextStore);
  const organizationId = $derived(context().organization.id ?? '');

  const canListAgents = $derived(can(Permission.LIST_AGENTS));
  const agentsQuery = createQuery(() => agentQueries.byOrganization(organizationId));
  const runPipeline = createMutation(() => pipelineMutations.run());

  const inFlight = new SvelteSet<string>();

  const announce = () => {
    if (!canListAgents) {
      toast.success(i18n._(ToastMessages.PIPELINE_RUN_CHECK_AGENTS));
      return;
    }

    if (!(agentsQuery.data ?? []).some(agent => agent.connected)) {
      toast.warning(i18n._(ToastMessages.PIPELINE_JOB_QUEUED_WARNING), {
        action: { label: 'Agents', onClick: () => scyllaNavigate.goToOrgRoute('/agents') },
      });
      return;
    }

    toast.success(i18n._(ToastMessages.PIPELINE_RUN));
  };

  return {
    isRunning: (pipelineId: string) => inFlight.has(pipelineId),

    async run(pipelineId: string): Promise<void> {
      inFlight.add(pipelineId);
      try {
        await runPipeline.mutateAsync(pipelineId);
        announce();
      } catch {
        // The global mutation handler toasts the error.
      } finally {
        inFlight.delete(pipelineId);
      }
    },
  };
};

export type RunPipeline = ReturnType<typeof createRunPipeline>;
