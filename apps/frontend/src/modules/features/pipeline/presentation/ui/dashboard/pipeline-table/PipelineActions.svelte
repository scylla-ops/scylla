<script lang="ts">
  import CopyIcon from '@lucide/svelte/icons/copy';
  import ListChecksIcon from '@lucide/svelte/icons/list-checks';
  import Loader2Icon from '@lucide/svelte/icons/loader-2';
  import MoreHorizontalIcon from '@lucide/svelte/icons/more-horizontal';
  import PencilIcon from '@lucide/svelte/icons/pencil';
  import PlayIcon from '@lucide/svelte/icons/play';
  import ZapIcon from '@lucide/svelte/icons/zap';
  import { Permission, can } from '@platform/authz';
  import {
    Button,
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from '@shadcn';
  import { IconButton } from '@shared/presentation/ui';
  import { createCompactContainer } from '@shared/presentation/state/compact-container.svelte.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { pipelineMessages } from '../../../pipeline.messages.ts';

  interface Props {
    onRun: (event: MouseEvent) => void;
    onEdit: (event: MouseEvent) => void;
    onDuplicate: (event: MouseEvent) => void;
    onViewJobs?: (event: MouseEvent) => void;
    onViewTriggers?: (event: MouseEvent) => void;
    isRunning?: boolean;
    isDuplicating?: boolean;
  }

  let {
    onRun,
    onEdit,
    onDuplicate,
    onViewJobs,
    onViewTriggers,
    isRunning = false,
    isDuplicating = false,
  }: Props = $props();

  const compact = createCompactContainer();

  // Gate each action by the permission it needs, in the current project context.
  const canRun = $derived(can(Permission.RUN_PIPELINE));
  const canEdit = $derived(can(Permission.UPDATE_PIPELINE));
  const canDuplicate = $derived(can(Permission.CREATE_PIPELINE));
  const showTriggers = $derived(!!onViewTriggers && can(Permission.MANAGE_TRIGGERS));
</script>

<!--
  A pipeline row's actions, laid out inline and collapsing into a dropdown once
  the column is too narrow for them. The breakpoint is this element's own width
  — a table column is sized independently of the viewport — which is why it is
  measured with an action rather than read from a media query.
-->
<div
  use:compact.measure
  class="flex w-full shrink-0 items-center justify-center gap-2"
>
  {#if compact.isCompact}
    <DropdownMenu>
      <DropdownMenuTrigger>
        {#snippet child({ props })}
          <Button
            {...props}
            type="button"
            size="icon"
            variant="ghost"
            class="h-8 w-8 shrink-0 rounded-full text-slate-400 hover:text-slate-900"
          >
            <MoreHorizontalIcon class="h-4 w-4" />
            <!-- The React trigger had no accessible name at all: an icon-only
                 button whose only label was the icon itself. -->
            <span class="sr-only">{t(pipelineMessages.pipelineActions)}</span>
          </Button>
        {/snippet}
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" class="w-40">
        {#if canRun}
          <DropdownMenuItem onclick={onRun}>
            <PlayIcon class="mr-2 h-4 w-4" />
            {t(pipelineMessages.run)}
          </DropdownMenuItem>
        {/if}
        {#if canEdit}
          <DropdownMenuItem onclick={onEdit}>
            <PencilIcon class="mr-2 h-4 w-4" />
            {t(pipelineMessages.edit)}
          </DropdownMenuItem>
        {/if}
        {#if canDuplicate}
          <DropdownMenuItem onclick={onDuplicate}>
            <CopyIcon class="mr-2 h-4 w-4" />
            {t(pipelineMessages.duplicate)}
          </DropdownMenuItem>
        {/if}
        {#if onViewJobs}
          <DropdownMenuItem onclick={onViewJobs}>
            <ListChecksIcon class="mr-2 h-4 w-4" />
            {t(pipelineMessages.viewJobs)}
          </DropdownMenuItem>
        {/if}
        {#if showTriggers}
          <DropdownMenuItem onclick={onViewTriggers}>
            <ZapIcon class="mr-2 h-4 w-4" />
            {t(pipelineMessages.triggers)}
          </DropdownMenuItem>
        {/if}
      </DropdownMenuContent>
    </DropdownMenu>
  {:else}
    {#if canRun}
      <IconButton
        icon={isRunning ? Loader2Icon : PlayIcon}
        tooltip={t(pipelineMessages.run)}
        onclick={onRun}
        busy={isRunning}
        iconClass={isRunning ? 'animate-spin' : 'fill-current'}
      />
    {/if}

    {#if canEdit}
      <IconButton
        icon={PencilIcon}
        tooltip={t(pipelineMessages.editPipeline)}
        onclick={onEdit}
      />
    {/if}

    {#if canDuplicate}
      <IconButton
        icon={isDuplicating ? Loader2Icon : CopyIcon}
        tooltip={t(pipelineMessages.duplicate)}
        onclick={onDuplicate}
        busy={isDuplicating}
        iconClass={isDuplicating ? 'animate-spin' : undefined}
      />
    {/if}

    {#if onViewJobs}
      <IconButton
        icon={ListChecksIcon}
        tooltip={t(pipelineMessages.viewJobs)}
        onclick={onViewJobs}
      />
    {/if}

    {#if showTriggers}
      <IconButton icon={ZapIcon} tooltip={t(pipelineMessages.triggers)} onclick={onViewTriggers} />
    {/if}
  {/if}
</div>
