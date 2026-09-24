<script lang="ts">
  import { SvelteSet } from 'svelte/reactivity';
  import ClockIcon from '@lucide/svelte/icons/clock';
  import HelpCircleIcon from '@lucide/svelte/icons/help-circle';
  import WebhookIcon from '@lucide/svelte/icons/webhook';
  import { scyllaNavigate } from '@platform/context';
  import { createMutation } from '@platform/query';
  import { Switch } from '@shadcn';
  import {
    ConfirmOperationAlertDialog,
    CopyableText,
    DataTable,
  } from '@shared/presentation/ui';
  import { createSelection } from '@shared/presentation/state/selection.svelte.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { TriggerEntity } from '../../../domain/entities/trigger.entity.ts';
  import { TriggerKind } from '../../../domain/structs/trigger-source.struct.ts';
  import { triggerMutations } from '../../triggers.queries.ts';
  import TriggerActions from './TriggerActions/TriggerActions.svelte';
  import TriggerSourceCell from './TriggerSourceCell/TriggerSourceCell.svelte';
  import TriggerStatusCell from './TriggerStatusCell/TriggerStatusCell.svelte';
  import TriggerFormDialog from '../dialogs/TriggerFormDialog/TriggerFormDialog.svelte';
  import { triggerColumns } from './trigger-columns.ts';
  import { triggersMessages } from '../triggers.messages.ts';

  interface Props {
    triggers: TriggerEntity[];
    pipelineId: string;
    pipelineName: string;
  }

  let { triggers, pipelineId, pipelineName }: Props = $props();

  const SOURCE_ICON = {
    [TriggerKind.Cron]: ClockIcon,
    [TriggerKind.Webhook]: WebhookIcon,
    [TriggerKind.Unknown]: HelpCircleIcon,
  } as const;

  const selection = createSelection('triggers');
  const deleteTrigger = createMutation(() => triggerMutations.remove(pipelineId));
  const setEnabled = createMutation(() => triggerMutations.setEnabled(pipelineId));
  const fireNow = createMutation(() => triggerMutations.fireNow(pipelineId));

  let editTarget = $state<TriggerEntity | null>(null);
  let deleteTargetId = $state<string | null>(null);
  /** Per-row, because two rows can be firing at once. */
  const firingIds = new SvelteSet<string>();

  const handleFire = (trigger: TriggerEntity) => {
    firingIds.add(trigger.id);

    fireNow
      .mutateAsync(trigger.id)
      // Close the loop: land on the run we just created.
      .then(() => scyllaNavigate.goToJobs(pipelineId, pipelineName))
      .catch(() => {
        // Toast shown by the global MutationCache onError handler.
      })
      .finally(() => {
        firingIds.delete(trigger.id);
      });
  };

  const columns = $derived(
    triggerColumns({
      name: nameCell,
      source: sourceCell,
      status: statusCell,
      enabled: enabledCell,
      actions: actionsCell,
    }),
  );
</script>

{#snippet nameCell(trigger: TriggerEntity)}
  {@const Icon = SOURCE_ICON[trigger.source.kind]}
  <div class="flex w-full min-w-0 flex-row items-center gap-3">
    <div class="flex size-10 shrink-0 items-center justify-center rounded-lg bg-primary/10">
      <Icon class="size-4 text-primary" />
    </div>
    <div class="flex min-w-0 flex-col text-start">
      <p class="truncate font-semibold text-foreground">{trigger.name}</p>
      <CopyableText
        class="text-xs text-muted-foreground/80"
        value={trigger.id}
        display={`ID: ${trigger.id}`}
      />
    </div>
  </div>
{/snippet}

{#snippet sourceCell(trigger: TriggerEntity)}
  <TriggerSourceCell {trigger} />
{/snippet}

{#snippet statusCell(trigger: TriggerEntity)}
  <TriggerStatusCell {trigger} />
{/snippet}

{#snippet enabledCell(trigger: TriggerEntity)}
  <!-- Stop propagation so toggling the switch doesn't also select the row. -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="flex justify-center" onclick={event => event.stopPropagation()}>
    <Switch
      checked={trigger.enabled}
      onCheckedChange={checked => setEnabled.mutate({ triggerId: trigger.id, enabled: checked })}
      aria-label={t(triggersMessages.toggleEnabled(trigger.name))}
    />
  </div>
{/snippet}

{#snippet actionsCell(trigger: TriggerEntity)}
  <TriggerActions
    isFiring={firingIds.has(trigger.id)}
    onFire={event => {
      event.stopPropagation();
      handleFire(trigger);
    }}
    onEdit={event => {
      event.stopPropagation();
      editTarget = trigger;
    }}
    onDelete={event => {
      event.stopPropagation();
      deleteTargetId = trigger.id;
    }}
  />
{/snippet}

<DataTable
  {columns}
  data={triggers}
  onRowClick={row => selection.select(row.original.id)}
  getRowId={trigger => trigger.id}
  isRowSelected={trigger => selection.selectedIds.includes(trigger.id)}
  alignColumnsCenter
/>

{#if editTarget}
  <TriggerFormDialog
    open
    onOpenChange={open => {
      if (!open) editTarget = null;
    }}
    {pipelineId}
    trigger={editTarget}
  />
{/if}

<ConfirmOperationAlertDialog
  open={deleteTargetId !== null}
  onOpenChange={open => {
    if (!open) deleteTargetId = null;
  }}
  onContinue={() => {
    if (!deleteTargetId) return;
    deleteTrigger.mutate(deleteTargetId);
    deleteTargetId = null;
  }}
/>
