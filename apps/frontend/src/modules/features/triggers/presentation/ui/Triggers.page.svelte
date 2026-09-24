<script lang="ts">
  import { contextStore } from '@platform/context';
  import { createQuery } from '@platform/query';
  import { ErrorState, SecretRevealDialog } from '@shared/presentation/ui';
  import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
  import { ScyllaError } from '@shared/utils/scylla-result.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { CreatedTrigger } from '../../domain/entities/trigger.entity.ts';
  import { triggerQueries } from '../triggers.queries.ts';
  import TriggersHeader from './components/TriggersHeader.svelte';
  import TriggersOverview from './components/TriggersOverview.svelte';
  import TriggerFormDialog from './dialogs/TriggerFormDialog.svelte';
  import TriggersTable from './triggers-table/TriggersTable.svelte';
  import { triggersMessages } from './triggers.messages.ts';

  interface Props {
    /** From the route: `/:organizationSlug/projects/:projectId/pipelines/:pipelineId/triggers`. */
    pipelineId?: string;
    projectId?: string;
  }

  let { pipelineId, projectId }: Props = $props();

  const context = toRune(contextStore);
  const pipelineName = $derived(context().pipeline?.name ?? '');

  const triggersQuery = createQuery(() => triggerQueries.byPipeline(pipelineId ?? ''));
  const triggers = $derived(triggersQuery.data ?? []);

  let isCreateOpen = $state(false);
  /** The one-time webhook secret, held only as long as the dialog shows it. */
  let revealed = $state<{ id: string; name: string; secret: string } | null>(null);

  const errorMessage = $derived(
    triggersQuery.error instanceof ScyllaError
      ? triggersQuery.error.userMessage()
      : t(triggersMessages.loadError),
  );

  const handleCreated = (created: CreatedTrigger) => {
    // The webhook secret is returned exactly once — reveal it now or it's lost.
    if (!created.webhookSecret) return;

    revealed = {
      id: created.trigger.id,
      name: created.trigger.name,
      secret: created.webhookSecret,
    };
  };
</script>

{#if !pipelineId || !projectId}
  <ErrorState message={t(triggersMessages.pipelineIdMissing)} />
{:else if triggersQuery.isLoading}
  <!-- Nothing: the list is the page, and a skeleton of it would be the page. -->
{:else if triggersQuery.isError}
  <ErrorState message={errorMessage} />
{:else}
  <div class="flex flex-col gap-4 w-full min-h-full">
    <TriggersHeader
      count={triggers.length}
      triggerIds={triggers.map(trigger => trigger.id)}
      {pipelineId}
      onNew={() => (isCreateOpen = true)}
    />

    {#if triggers.length > 0}
      <TriggersOverview {triggers} />
    {/if}

    <div class="overflow-hidden">
      {#if triggers.length > 0}
        <TriggersTable {triggers} {pipelineId} {pipelineName} />
      {:else}
        <div class="flex items-center justify-center min-h-[300px]">
          <div class="text-center space-y-2">
            <p class="text-muted-foreground">{t(triggersMessages.noTriggersYet)}</p>
            <p class="text-sm text-muted-foreground">{t(triggersMessages.noTriggersBody)}</p>
          </div>
        </div>
      {/if}
    </div>

    <TriggerFormDialog
      open={isCreateOpen}
      onOpenChange={open => (isCreateOpen = open)}
      {pipelineId}
      onCreated={handleCreated}
    />

    {#if revealed}
      <SecretRevealDialog
        open
        title={t(triggersMessages.webhookSecretOf(revealed.name))}
        description={t(triggersMessages.copySecretOnce)}
        secret={revealed.secret}
        secretLabel="Secret"
        revealedNote={t(triggersMessages.addSigningSecret)}
        onClose={() => (revealed = null)}
      />
    {/if}
  </div>
{/if}
