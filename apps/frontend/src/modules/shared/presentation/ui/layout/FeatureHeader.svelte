<script lang="ts">
  import type { Snippet } from 'svelte';
  import TrashIcon from '@lucide/svelte/icons/trash';
  import { Button } from '@shadcn';
  import { toast } from '@shared/presentation/utils/toast.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  // Direct paths, not the group barrels: importing them from here would loop
  // back through `layout`.
  import GatedButton from '../controls/GatedButton.svelte';
  import ConfirmOperationAlertDialog from '../feedback/ConfirmOperationAlertDialog.svelte';
  import { featureHeaderMessages } from './feature-header.messages.ts';

  interface Props {
    count?: number;
    label: string;
    underLabel?: Snippet;
    pluralLabel?: string;
    selectedCount?: number;
    /** True when every selectable row is already selected — hides "Select all". */
    allSelected?: boolean;
    onSelectAll?: () => void;
    onClearSelection?: () => void;
    onDeleteSelection?: () => Promise<void> | void;
    onNew?: () => void;
    newLabel?: string;
    /** When false, the "New" button is shown disabled with {@link newDeniedReason}. */
    canNew?: boolean;
    newDeniedReason?: string;
    /** When false, the bulk-delete button is shown disabled with {@link deleteDeniedReason}. */
    canDelete?: boolean;
    deleteDeniedReason?: string;
    extraActions?: Snippet;
  }

  let {
    count,
    label,
    underLabel,
    pluralLabel,
    selectedCount = 0,
    allSelected = false,
    onSelectAll,
    onClearSelection,
    onDeleteSelection,
    onNew,
    newLabel,
    canNew = true,
    newDeniedReason,
    canDelete = true,
    deleteDeniedReason,
    extraActions,
  }: Props = $props();

  let deleteDialogOpen = $state(false);

  const displayLabel = $derived(count && count > 1 ? (pluralLabel ?? label) : label);
  const newButtonLabel = $derived(newLabel ?? t(featureHeaderMessages.newEntity(label)));

  const handleDelete = async () => {
    try {
      deleteDialogOpen = false;
      await onDeleteSelection?.();
      toast.success(t(featureHeaderMessages.itemsDeleted(selectedCount)));
    } catch {
      // Toast shown by the global MutationCache onError handler.
      deleteDialogOpen = false;
    }
  };
</script>

<div class="flex w-full flex-row flex-wrap items-end justify-between gap-x-4 gap-y-3">
  <div class="flex min-w-0 flex-col gap-2">
    <div class="flex flex-row flex-wrap gap-4">
      <div class="flex min-w-0 items-baseline gap-2">
        <h1 class="min-w-0 text-2xl font-bold tracking-tight break-words sm:text-3xl">
          {#if count !== undefined}<span class="mr-2 text-primary">{count}</span>{/if}
          <span class="text-foreground">{displayLabel}</span>
        </h1>
        {#if count !== undefined}
          <span class="text-sm font-medium whitespace-nowrap text-muted-foreground">
            {t(featureHeaderMessages.inTotal)}
          </span>
        {/if}
      </div>
    </div>
    {#if underLabel}{@render underLabel()}{/if}
  </div>

  <!-- Wraps under the title rather than pushing the actions out of the page. -->
  <div class="ml-auto flex flex-wrap items-center justify-end gap-2">
    {#if onSelectAll && !allSelected && !!count}
      <Button variant="outline" onclick={onSelectAll}>{t(featureHeaderMessages.selectAll)}</Button>
    {/if}

    {#if selectedCount > 0 && onClearSelection}
      <Button variant="outline" onclick={onClearSelection}>{t(featureHeaderMessages.clear)}</Button>
    {/if}

    {#if selectedCount > 0 && onDeleteSelection}
      <GatedButton
        allowed={canDelete}
        deniedReason={deleteDeniedReason}
        tooltip={t(featureHeaderMessages.delete)}
        size="icon"
        variant="destructive"
        onclick={() => (deleteDialogOpen = true)}
        class="h-9 w-9 cursor-pointer transition-all hover:scale-110"
      >
        <TrashIcon class="size-4" />
        <span class="sr-only">{t(featureHeaderMessages.delete)}</span>
      </GatedButton>
    {/if}

    {#if extraActions}{@render extraActions()}{/if}

    {#if onNew}
      <GatedButton allowed={canNew} deniedReason={newDeniedReason} onclick={onNew}>
        {newButtonLabel}
      </GatedButton>
    {/if}
  </div>

  {#if onDeleteSelection}
    <ConfirmOperationAlertDialog
      open={deleteDialogOpen}
      onOpenChange={next => (deleteDialogOpen = next)}
      onContinue={handleDelete}
    />
  {/if}
</div>
