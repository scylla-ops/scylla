import { i18n } from '@lingui/core';
import { SvelteSet } from 'svelte/reactivity';
import { Permission, can } from '@platform/authz';
import { scyllaNavigate, useContextStore } from '@platform/context';
import { createMutation, createQuery } from '@platform/query';
import { agentQueries } from '@/modules/features/agents';
import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import { pipelineMutations } from './pipeline.queries.ts';

/**
 * Starting a run, and saying what is likely to happen next.
 *
 * The run itself always succeeds — the job is created and queued — so the
 * interesting part is the follow-up: with no connected agent the job will sit
 * there, and saying so up front beats letting the user watch a spinner. That
 * needs the agent list, which is why this is a ViewModel and not part of
 * `pipelineMutations`: the advice is only honest if something is subscribed to
 * the agents.
 *
 * Without `LIST_AGENTS` the list is never fetched — asking would only be denied
 * — so connectivity is genuinely unknown. The message then points at agents as
 * something to check, rather than claiming none is connected.
 */
export const createRunPipeline = () => {
  const context = toRune(useContextStore);
  const organizationId = $derived(context().organization.id ?? '');

  const canListAgents = $derived(can(Permission.LIST_AGENTS));
  const agentsQuery = createQuery(() => agentQueries.byOrganization(organizationId));
  const runPipeline = createMutation(() => pipelineMutations.run());

  /** Reactive because the table spins one button per row while it is in flight. */
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
        // Toast shown by the global MutationCache onError handler.
      } finally {
        inFlight.delete(pipelineId);
      }
    },
  };
};

export type RunPipeline = ReturnType<typeof createRunPipeline>;
