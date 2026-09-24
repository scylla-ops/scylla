<script lang="ts">
  import EditIcon from '@lucide/svelte/icons/pencil';
  import Loader2Icon from '@lucide/svelte/icons/loader-2';
  import MoreHorizontalIcon from '@lucide/svelte/icons/more-horizontal';
  import PlayIcon from '@lucide/svelte/icons/play';
  import Trash2Icon from '@lucide/svelte/icons/trash-2';
  import { can, Permission } from '@platform/authz';
  import {
    buttonVariants,
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from '@shadcn';
  import { IconButton } from '@shared/presentation/ui';
  import { createCompactContainer } from '@shared/presentation/state/compact-container.svelte.ts';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { triggersMessages } from '../triggers.messages.ts';

  interface Props {
    onFire: (event: Event) => void;
    onEdit: (event: Event) => void;
    onDelete: (event: Event) => void;
    isFiring?: boolean;
  }

  let { onFire, onEdit, onDelete, isFiring = false }: Props = $props();

  const canManage = $derived(can(Permission.MANAGE_TRIGGERS));

  // An action, not a ref + effect: this element lives in a table row and is
  // created and destroyed with it, which is the case the React pair kept
  // getting wrong.
  const container = createCompactContainer();
</script>

<!--
  Row actions: fire now (test), edit, delete. All three write to the trigger, so
  they hide together behind `MANAGE_TRIGGERS`.

  Switches to dropdown mode when the column gets too narrow for three buttons.
-->
{#if canManage}
  <div
    use:container.measure
    class="flex w-full shrink-0 items-center justify-center gap-1"
  >
    {#if container.isCompact}
      <DropdownMenu>
        <DropdownMenuTrigger
          class={cn(
            buttonVariants({ variant: 'ghost', size: 'icon' }),
            'h-8 w-8 shrink-0 rounded-full text-slate-400 hover:text-slate-900',
          )}
          onclick={(event: MouseEvent) => event.stopPropagation()}
        >
          <MoreHorizontalIcon class="h-4 w-4" />
          <!-- Icon-only: the React trigger carried no accessible name. -->
          <span class="sr-only">{t(triggersMessages.triggerActions)}</span>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" class="w-40">
          <DropdownMenuItem onSelect={onFire} disabled={isFiring}>
            <PlayIcon class="mr-2 h-4 w-4" />
            {t(triggersMessages.fireNow)}
          </DropdownMenuItem>
          <DropdownMenuItem onSelect={onEdit}>
            <EditIcon class="mr-2 h-4 w-4" />
            {t(triggersMessages.edit)}
          </DropdownMenuItem>
          <DropdownMenuItem onSelect={onDelete} class="text-destructive">
            <Trash2Icon class="mr-2 h-4 w-4" />
            {t(triggersMessages.delete)}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    {:else}
      <IconButton
        icon={isFiring ? Loader2Icon : PlayIcon}
        tooltip={t(triggersMessages.fireNow)}
        onclick={onFire}
        busy={isFiring}
        iconClass={isFiring ? 'animate-spin' : 'fill-current'}
      />
      <IconButton icon={EditIcon} tooltip={t(triggersMessages.edit)} onclick={onEdit} />
      <IconButton
        icon={Trash2Icon}
        tooltip={t(triggersMessages.delete)}
        onclick={onDelete}
        iconClass="text-destructive"
      />
    {/if}
  </div>
{/if}
