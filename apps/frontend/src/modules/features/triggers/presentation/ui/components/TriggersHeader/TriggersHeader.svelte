<script lang="ts">
  import { can, Permission } from '@platform/authz';
  import { createMutation } from '@platform/query';
  import { CopyableText, FeatureHeader } from '@shared/presentation/ui';
  import { createFeatureSelection } from '@shared/presentation/state/feature-selection.svelte.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { triggerMutations } from '../../../triggers.queries.ts';
  import { triggersMessages } from '../../triggers.messages.ts';

  interface Props {
    count: number;
    triggerIds: string[];
    pipelineId: string;
    onNew: () => void;
  }

  let { count, triggerIds, pipelineId, onNew }: Props = $props();

  const deleteTrigger = createMutation(() => triggerMutations.remove(pipelineId));

  // A getter, not the array: the ids are server data that arrives late, and
  // capturing them once would freeze "select all" on the first, empty render.
  const selection = createFeatureSelection('triggers', () => triggerIds, {
    deleteItem: id => deleteTrigger.mutateAsync(id),
  });

  // Triggers are all-or-nothing in V1: one permission covers create and delete.
  const canManage = $derived(can(Permission.MANAGE_TRIGGERS));
</script>

{#snippet pipelineIdLine()}
  <div class="flex items-center gap-2">
    <span class="text-sm text-muted-foreground font-medium">
      {t(triggersMessages.pipelineIdLabel)}
    </span>
    <CopyableText value={pipelineId} class="text-sm text-muted-foreground font-medium" />
  </div>
{/snippet}

<FeatureHeader
  {count}
  label={t(triggersMessages.trigger)}
  pluralLabel={t(triggersMessages.triggers)}
  newLabel={t(triggersMessages.newTrigger)}
  {onNew}
  canNew={canManage}
  newDeniedReason={t(triggersMessages.manageDenied)}
  canDelete={canManage}
  deleteDeniedReason={t(triggersMessages.manageDenied)}
  underLabel={pipelineIdLine}
  {...selection.headerProps}
/>
