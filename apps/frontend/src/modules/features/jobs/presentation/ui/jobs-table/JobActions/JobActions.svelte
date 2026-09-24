<script lang="ts">
  import EyeIcon from '@lucide/svelte/icons/eye';
  import MoreHorizontalIcon from '@lucide/svelte/icons/more-horizontal';
  import TrashIcon from '@lucide/svelte/icons/trash';
  import { Permission, can } from '@platform/authz';
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
  import { jobsMessages } from '../../jobs.messages.ts';

  interface Props {
    onView: (event: Event) => void;
    onDelete: (event: Event) => void;
  }

  let { onView, onDelete }: Props = $props();

  const canDelete = $derived(can(Permission.DELETE_JOB));

  const container = createCompactContainer();
</script>

<!-- Falls back to a menu when the column is too narrow. -->
<div use:container.measure class="flex w-full items-center justify-center gap-2 shrink-0">
  {#if container.isCompact}
    <DropdownMenu>
      <DropdownMenuTrigger
        class={cn(
          buttonVariants({ variant: 'ghost', size: 'icon' }),
          'h-8 w-8 shrink-0 rounded-full text-slate-400 hover:text-slate-900',
        )}
        onclick={(event: MouseEvent) => event.stopPropagation()}
      >
        <MoreHorizontalIcon class="w-4 h-4" />
        <span class="sr-only">{t(jobsMessages.jobActions)}</span>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" class="w-40">
        <DropdownMenuItem onSelect={onView}>
          <EyeIcon class="w-4 h-4 mr-2" />
          {t(jobsMessages.view)}
        </DropdownMenuItem>
        {#if canDelete}
          <DropdownMenuItem onSelect={onDelete} class="text-destructive">
            <TrashIcon class="w-4 h-4 mr-2" />
            {t(jobsMessages.delete)}
          </DropdownMenuItem>
        {/if}
      </DropdownMenuContent>
    </DropdownMenu>
  {:else}
    <IconButton icon={EyeIcon} tooltip={t(jobsMessages.view)} onclick={onView} />

    {#if canDelete}
      <IconButton
        icon={TrashIcon}
        tooltip={t(jobsMessages.delete)}
        onclick={onDelete}
        class="hover:text-destructive hover:bg-destructive-subtle"
      />
    {/if}
  {/if}
</div>
