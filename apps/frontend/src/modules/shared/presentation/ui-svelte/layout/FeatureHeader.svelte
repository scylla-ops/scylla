<script lang="ts">
  import type { Snippet } from 'svelte';
  import TrashIcon from '@lucide/svelte/icons/trash';
  import { Button, Tooltip, TooltipContent, TooltipTrigger } from '@shadcn-svelte';
  import { cn } from '@shared/presentation/utils';
  import { toast } from '@shared/presentation/utils/toast.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
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
      <Tooltip>
        <!--
          A span between trigger and button, as in the React original: a disabled
          button fires no pointer events, so the tooltip explaining *why* it is
          disabled would never open if the trigger were the button itself.
        -->
        <TooltipTrigger>
          {#snippet child({ props })}
            <span {...props} class="inline-flex">
              <Button
                size="icon"
                variant="destructive"
                disabled={!canDelete}
                onclick={() => (deleteDialogOpen = true)}
                class={cn(
                  'h-9 w-9 cursor-pointer transition-all hover:scale-110',
                  !canDelete && 'pointer-events-none',
                )}
              >
                <TrashIcon class="size-4" />
                <span class="sr-only">{t(featureHeaderMessages.delete)}</span>
              </Button>
            </span>
          {/snippet}
        </TooltipTrigger>
        <TooltipContent>
          <p>
            {canDelete
              ? t(featureHeaderMessages.delete)
              : (deleteDeniedReason ?? t(featureHeaderMessages.notPermitted))}
          </p>
        </TooltipContent>
      </Tooltip>
    {/if}

    {#if extraActions}{@render extraActions()}{/if}

    {#if onNew}
      {#if canNew}
        <Button onclick={onNew}>{newButtonLabel}</Button>
      {:else}
        <Tooltip>
          <TooltipTrigger>
            {#snippet child({ props })}
              <span {...props} class="inline-flex">
                <Button disabled class="pointer-events-none">{newButtonLabel}</Button>
              </span>
            {/snippet}
          </TooltipTrigger>
          <TooltipContent>
            <p>{newDeniedReason ?? t(featureHeaderMessages.notPermitted)}</p>
          </TooltipContent>
        </Tooltip>
      {/if}
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
