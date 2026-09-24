<script lang="ts">
  import type { Snippet } from 'svelte';
  import CpuIcon from '@lucide/svelte/icons/cpu';
  import { can, Permission } from '@platform/authz';
  import { createResourceError, scyllaNavigate, contextStore } from '@platform/context';
  import { createMutation, createQuery } from '@platform/query';
  import { Badge, Button, Skeleton } from '@shadcn';
  import {
    AgentRunInstructions,
    ConfirmOperationAlertDialog,
    ErrorState,
  } from '@shared/presentation/ui';
  import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
  import { cn } from '@shared/presentation/utils';
  import { formatDate, getRelativeTime } from '@shared/utils/date-utils.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { agentMutations, agentQueries } from '../../agents.queries.ts';
  import AgentIdLink from '../components/AgentIdLink/AgentIdLink.svelte';
  import OutcomesChart from '../components/OutcomesChart.svelte';
  import { agentsMessages } from '../agents.messages.ts';

  interface Props {
    /** From the route: `/:organizationSlug/agents/:agentId`. */
    agentId?: string;
  }

  let { agentId }: Props = $props();

  const context = toRune(contextStore);
  const organizationId = $derived(context().organization.id ?? '');

  const agentQuery = createQuery(() => agentQueries.byId(agentId ?? ''));
  // Gated on READ_APP_STATS inside the factory; asked again here because the
  // whole section is hidden rather than shown empty.
  const canReadStats = $derived(can(Permission.READ_APP_STATS));
  const statsQuery = createQuery(() => agentQueries.statsOf(agentId ?? ''));
  const deleteAgent = createMutation(() => agentMutations.remove(organizationId));

  const agent = $derived(agentQuery.data);
  const stats = $derived(statsQuery.data);
  const online = $derived(agent?.connected ?? false);
  const seenLabel = $derived(agent?.lastSeen ? getRelativeTime(agent.lastSeen) : null);

  const canDelete = $derived(can(Permission.DELETE_APP));

  // NOT_FOUND (deleted / bad id) → toast + back to the agents list.
  const resourceError = createResourceError({
    error: () => agentQuery.error,
    redirectTo: '..',
    notFoundMessage: t(agentsMessages.notFound),
  });

  let confirmDelete = $state(false);

  const handleDelete = () => {
    if (!agent) return;
    deleteAgent.mutate(agent.id, { onSuccess: () => scyllaNavigate.navigate('..') });
    confirmDelete = false;
  };
</script>

{#snippet stripLabel(children: Snippet)}
  <span class="font-mono text-[10px] uppercase tracking-wide text-muted-foreground">
    {@render children()}
  </span>
{/snippet}

{#if resourceError.redirecting}
  <!-- Nothing: the redirect is already under way. -->
{:else if agentQuery.isLoading}
  <Skeleton class="m-4 h-72 rounded-xl" />
{:else if agentQuery.isError || !agent}
  <ErrorState message={t(agentsMessages.detailsLoadError)} />
{:else}
  <div class="w-full min-h-full flex flex-col gap-6 pb-8">
    <!-- Header -->
    <div class="flex items-center justify-between w-full gap-4">
      <div class="flex items-center gap-3">
        <span
          class={cn(
            'relative flex h-14 w-14 items-center justify-center rounded-lg bg-success/10',
            online ? 'border border-success' : 'border-2 border-destructive',
          )}
        >
          <CpuIcon class={cn('h-7 w-7', online ? 'text-success' : 'text-destructive')} />
          <span class="absolute -bottom-1 -right-1 flex h-3.5 w-3.5" aria-hidden="true">
            {#if online}
              <span
                class="absolute inline-flex h-full w-full animate-ping rounded-full bg-success/70"
              ></span>
            {/if}
            <span
              class={cn(
                'relative inline-flex h-3.5 w-3.5 rounded-full border-2 border-card',
                online ? 'bg-success' : 'bg-destructive',
              )}
            ></span>
          </span>
        </span>
        <div>
          <h1 class="text-xl font-semibold text-foreground">{agent.name}</h1>
          <p class="font-mono text-xs text-muted-foreground">
            {online ? t(agentsMessages.online) : t(agentsMessages.offline)}
            {#if seenLabel}
              ·
              {online ? t(agentsMessages.seen) : t(agentsMessages.down)}
              {seenLabel}
            {/if}
          </p>
        </div>
      </div>

      {#if canDelete}
        <Button variant="destructive" onclick={() => (confirmDelete = true)}>
          {t(agentsMessages.delete)}
        </Button>
      {/if}
    </div>

    <!-- Identity strip -->
    <div
      class="flex flex-wrap items-center gap-x-3 gap-y-1.5 rounded-md border border-border bg-card px-3.5 py-2 w-full"
    >
      <span class="flex items-center gap-1.5">
        {@render stripLabel(agentIdLabel)}
        <AgentIdLink id={agent.id} />
      </span>
      <span class="text-muted-foreground/50">·</span>
      <span class="flex items-center gap-1.5">
        {@render stripLabel(activeLabel)}
        <Badge variant={agent.isActive ? 'default' : 'secondary'}>
          {agent.isActive ? t(agentsMessages.active) : t(agentsMessages.inactive)}
        </Badge>
      </span>
      <span class="text-muted-foreground/50">·</span>
      <span class="flex items-center gap-1.5">
        {@render stripLabel(createdLabel)}
        <span class="font-mono text-xs text-foreground">{formatDate(agent.createdAt)}</span>
      </span>
      <span class="text-muted-foreground/50">·</span>
      <span class="flex items-center gap-1.5">
        {@render stripLabel(updatedLabel)}
        <span class="font-mono text-xs text-foreground">{formatDate(agent.updatedAt)}</span>
      </span>
    </div>

    {#if canReadStats}
      <div class="w-full">
        <div class="mb-2 flex items-baseline gap-2">
          <h2 class="text-lg font-semibold text-foreground">{t(agentsMessages.jobStats)}</h2>
          {#if stats}
            <span class="font-mono text-xs text-muted-foreground">
              {t(agentsMessages.since)}
              {formatDate(agent.createdAt)}
              {#if stats.lastRunAt}
                ·
                {t(agentsMessages.lastRun)}
                {getRelativeTime(stats.lastRunAt)}
              {/if}
            </span>
          {/if}
        </div>

        <div class="flex w-full gap-3">
          {#if stats}
            <OutcomesChart
              daily={stats.daily}
              aggregate={{
                completed: stats.completed,
                failed: stats.failed,
                cancelled: stats.cancelled,
              }}
            />
          {:else}
            <Skeleton class="h-64 w-full rounded-xl" />
          {/if}
        </div>
      </div>
    {/if}

    <!-- How to start a worker for this agent -->
    <div class="w-full">
      <div class="mb-2 flex items-baseline gap-2">
        <h2 class="text-lg font-semibold text-foreground">{t(agentsMessages.runThisAgent)}</h2>
        <span class="font-mono text-xs text-muted-foreground">{t(agentsMessages.connectsAs)}</span>
      </div>
      <div class="w-full">
        <AgentRunInstructions appId={agent.id} />
      </div>
    </div>

    <ConfirmOperationAlertDialog
      open={confirmDelete}
      onOpenChange={open => (confirmDelete = open)}
      onContinue={handleDelete}
      title={t(agentsMessages.deleteAgentTitle)}
      description={t(agentsMessages.deleteAgentBody)}
    />
  </div>
{/if}

{#snippet agentIdLabel()}{t(agentsMessages.agentId)}{/snippet}
{#snippet activeLabel()}{t(agentsMessages.activeLabel)}{/snippet}
{#snippet createdLabel()}{t(agentsMessages.createdOn)}{/snippet}
{#snippet updatedLabel()}{t(agentsMessages.updatedOn)}{/snippet}
