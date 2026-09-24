<script lang="ts">
  import type { Snippet } from 'svelte';
  import { can, Permission } from '@platform/authz';
  import { scyllaNavigate, contextStore } from '@platform/context';
  import { createQuery } from '@platform/query';
  import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { agentQueries } from '../../agents.queries.ts';
  import { agentsMessages } from '../agents.messages.ts';

  interface Props {
    /** Only warn when something is actually stuck behind the missing agent. */
    hasPendingJobs: boolean;
  }

  let { hasPendingJobs }: Props = $props();

  const context = toRune(contextStore);
  const organizationId = $derived(context().organization.id ?? '');

  // The query is gated on LIST_AGENTS inside `agentQueries`; this is the same
  // question asked again, because the banner must *say something different*
  // when it cannot look rather than claim no agent is connected.
  const canListAgents = $derived(can(Permission.LIST_AGENTS));

  const agentsQuery = createQuery(() => agentQueries.byOrganization(organizationId));
  const agents = $derived(agentsQuery.data ?? []);
  const anyConnected = $derived(agents.some(agent => agent.connected));
</script>

<!--
  Quiet inline banner shown when jobs are queued but no agent of the org is
  connected — without it a pending job is just a spinner that never moves.
  Disappears on its own once an agent comes online (the agents query refetches
  every 10s).

  Without LIST_AGENTS the agent list is never fetched, so connectivity is
  unknowable from here: the banner then only points at agents as the likely
  cause instead of asserting none is connected.
-->
{#snippet banner(children: Snippet)}
  <div
    class="flex items-center gap-2.5 rounded-md border px-3 py-2 text-xs text-muted-foreground"
  >
    <span class="h-1.5 w-1.5 shrink-0 animate-pulse rounded-full bg-warning"></span>
    {@render children()}
  </div>
{/snippet}

{#if hasPendingJobs && !agentsQuery.isLoading}
  {#if !canListAgents}
    {@render banner(cannotLook)}
  {:else if !anyConnected}
    {@render banner(noneConnected)}
  {/if}
{/if}

{#snippet cannotLook()}
  <span>{t(agentsMessages.jobsQueuedCheckAgents)}</span>
{/snippet}

{#snippet noneConnected()}
  <span>{t(agentsMessages.noAgentConnected)}</span>
  <button
    type="button"
    onclick={() => scyllaNavigate.goToOrgRoute('/agents')}
    class="ml-auto shrink-0 font-medium text-primary hover:underline"
  >
    {t(agentsMessages.setUpAnAgent)} →
  </button>
{/snippet}
