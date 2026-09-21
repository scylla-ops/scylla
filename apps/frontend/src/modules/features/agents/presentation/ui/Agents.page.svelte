<script lang="ts">
  import CpuIcon from '@lucide/svelte/icons/cpu';
  import PlusIcon from '@lucide/svelte/icons/plus';
  import { can, Permission } from '@platform/authz';
  import { scyllaNavigate, useContextStore } from '@platform/context';
  import { createMutation, createQuery } from '@platform/query';
  import { Button, Card, Skeleton } from '@shadcn-svelte';
  import {
    AgentRunInstructions,
    ConfirmOperationAlertDialog,
    ErrorState,
    FeatureHeader,
    FormDialog,
    SecretRevealDialog,
    type FormValues,
  } from '@shared/presentation/ui-svelte';
  import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
  import { activeLocale, t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { CreatedAgent } from '../../domain/structs/agent.struct.ts';
  import { agentMutations, agentQueries } from '../agents.queries.ts';
  import { createAgentItems } from '../utils/create-agent-form-items.ts';
  import AgentCard from './components/AgentCard.svelte';
  import { agentsMessages } from './agents.messages.ts';

  const context = toRune(useContextStore);
  const organizationId = $derived(context().organization.id ?? '');

  const agentsQuery = createQuery(() => agentQueries.byOrganization(organizationId));
  const createAgent = createMutation(() => agentMutations.create(organizationId));
  const deleteAgent = createMutation(() => agentMutations.remove(organizationId));

  const agents = $derived(agentsQuery.data ?? []);
  const onlineCount = $derived(agents.filter(agent => agent.connected).length);

  const canCreate = $derived(can(Permission.CREATE_AGENT));

  let createOpen = $state(false);
  /** The one-time plaintext, held only as long as the dialog shows it. */
  let created = $state<CreatedAgent | null>(null);
  /** The agent awaiting confirmation. `null` closes the dialog. */
  let pendingDeletion = $state<string | null>(null);

  const items = $derived((activeLocale(), createAgentItems()));

  const handleCreate = ({ name }: FormValues<'name'>) => {
    if (!name.trim()) return;

    // On error the dialog stays open so the name can be fixed in place; the
    // toast comes from the global MutationCache onError handler.
    createAgent.mutate(name.trim(), {
      onSuccess: data => {
        createOpen = false;
        created = data;
      },
    });
  };

  const confirmDelete = () => {
    if (pendingDeletion) deleteAgent.mutate(pendingDeletion);
    pendingDeletion = null;
  };

  const openCreated = () => {
    const agentId = created?.agent.id;
    created = null;
    if (agentId) scyllaNavigate.goToSubRoute(agentId);
  };
</script>

{#snippet onlineSummary()}
  {#if !agentsQuery.isLoading && agents.length > 0}
    <p class="mt-1 flex items-center gap-3 font-mono text-xs text-muted-foreground">
      <span class="flex items-center gap-1.5">
        <span class="h-2 w-2 rounded-full bg-success"></span>
        {onlineCount}
        {t(agentsMessages.online)}
      </span>
      <span class="flex items-center gap-1.5">
        <span class="h-2 w-2 rounded-full bg-destructive"></span>
        {agents.length - onlineCount}
        {t(agentsMessages.offline)}
      </span>
    </p>
  {/if}
{/snippet}

{#snippet runInstructions()}
  {#if created}
    <AgentRunInstructions appId={created.agent.id} secret={created.secret} />
  {/if}
{/snippet}

{#if agentsQuery.isError}
  <ErrorState message={t(agentsMessages.loadError)} />
{:else}
  <div class="flex flex-col gap-4 w-full h-full">
    <FeatureHeader
      count={agents.length}
      label={t(agentsMessages.agent)}
      pluralLabel={t(agentsMessages.agents)}
      onNew={() => (createOpen = true)}
      newLabel={t(agentsMessages.newAgent)}
      canNew={canCreate}
      newDeniedReason={t(agentsMessages.createDenied)}
      underLabel={onlineSummary}
    />

    {#if agentsQuery.isLoading}
      <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {#each { length: 6 } as _, index (index)}
          <Skeleton class="h-40 w-full rounded-xl" />
        {/each}
      </div>
    {:else if agents.length === 0}
      <div class="flex flex-1 items-center justify-center">
        <Card class="flex max-w-sm flex-col items-center gap-3 p-8 text-center">
          <span
            class="relative flex h-14 w-14 items-center justify-center rounded-xl border border-dashed border-muted-foreground/40 bg-muted/30"
          >
            <CpuIcon class="h-6 w-6 text-muted-foreground" />
            <span
              class="absolute -bottom-1 -right-1 h-3 w-3 rounded-full border-2 border-card bg-muted-foreground/50"
            ></span>
          </span>
          <h2 class="text-lg font-semibold">{t(agentsMessages.noAgentsConnected)}</h2>
          <p class="text-sm text-muted-foreground">{t(agentsMessages.noAgentsBody)}</p>
          {#if canCreate}
            <Button onclick={() => (createOpen = true)}>
              {t(agentsMessages.createFirstAgent)}
            </Button>
          {/if}
        </Card>
      </div>
    {:else}
      <div class="grid gap-3 p-2 overflow-y-auto sm:grid-cols-2 lg:grid-cols-3">
        {#each agents as agent (agent.id)}
          <AgentCard {agent} onRequestDelete={id => (pendingDeletion = id)} />
        {/each}
        {#if canCreate}
          <button
            type="button"
            onclick={() => (createOpen = true)}
            class="flex min-h-40 items-center justify-center gap-2 rounded-xl border border-dashed text-sm text-muted-foreground transition-colors hover:bg-accent/50 hover:text-foreground"
          >
            <PlusIcon class="h-4 w-4" />
            {t(agentsMessages.newAgent)}
          </button>
        {/if}
      </div>
    {/if}

    <FormDialog
      open={createOpen}
      onOpenChange={open => (createOpen = open)}
      title={t(agentsMessages.newAgent)}
      description={t(agentsMessages.getCredentials)}
      {items}
      isPending={createAgent.isPending}
      submitLabel={t(agentsMessages.createAndReveal)}
      onSubmit={handleCreate}
    />

    {#if created}
      <SecretRevealDialog
        open
        title={t(agentsMessages.agentIsReady(created.agent.name))}
        description={t(agentsMessages.twoSteps)}
        secret={created.secret}
        secretLabel={t(agentsMessages.secretForAgent)}
        secondStep={{ title: t(agentsMessages.startYourAgent), content: runInstructions }}
        onClose={openCreated}
      />
    {/if}

    <ConfirmOperationAlertDialog
      open={pendingDeletion !== null}
      onOpenChange={open => {
        if (!open) pendingDeletion = null;
      }}
      onContinue={confirmDelete}
      title={t(agentsMessages.deleteAgentTitle)}
      description={t(agentsMessages.deleteAgentBody)}
    />
  </div>
{/if}
