<script lang="ts">
  import { SvelteSet } from 'svelte/reactivity';
  import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
  import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
  import RadioIcon from '@lucide/svelte/icons/radio';
  import TerminalIcon from '@lucide/svelte/icons/terminal';
  import XIcon from '@lucide/svelte/icons/x';
  import { Permission, can } from '@platform/authz';
  import { Badge, Button } from '@shadcn';
  import { getStatusIcon } from '@shared/presentation/ui';
  import { createMeasuredHeight } from '@shared/presentation/state/measured-height.svelte.ts';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { calculateExecutionDuration, formatDuration } from '@shared/utils/date-utils.ts';
  import { getStatusConfig } from '@shared/utils/status-config.ts';
  import type { JobEntity } from '../../../domain/entities/job.entity.ts';
  import { nodeIdOf } from '../jobs-table/job-timeline.calculator.ts';
  import JobLogDisplay from '../jobs-log/JobLogDisplay/JobLogDisplay.svelte';
  import { jobsMessages } from '../jobs.messages.ts';

  const PANEL_HEADER_HEIGHT = 36;
  const PANEL_BORDER_HEIGHT = 2;
  /** The column scrolls rather than shrink a log below this. */
  const MIN_LOG_HEIGHT = 192;
  /** A node log's height, however many are open: the column scrolls. */
  const NODE_LOG_HEIGHT = 448;

  interface Props {
    job: JobEntity;
    /** From the URL, in execution order. */
    openNodeIds: readonly string[];
    isWholeJobOpen: boolean;
    onToggleNode: (nodeId: string) => void;
    /** Closes every node panel, which shows the whole job again. */
    onShowWholeJob: () => void;
  }

  let { job, openNodeIds, isWholeJobOpen, onToggleNode, onShowWholeJob }: Props = $props();

  const canViewLogs = $derived(can(Permission.READ_JOB_LOGS));

  // Sized by the layout, never by the panels, or the measure feeds back into itself.
  const column = createMeasuredHeight();

  const collapsedIds = new SvelteSet<string>();

  const toggleCollapse = (nodeId: string) => {
    if (collapsedIds.has(nodeId)) collapsedIds.delete(nodeId);
    else collapsedIds.add(nodeId);
  };

  const nodes = $derived(
    job.nodeExecutions.map((node, index) => ({ node, id: nodeIdOf(node, index) })),
  );
  const openNodes = $derived(nodes.filter(({ id }) => openNodeIds.includes(id)));

  /** The whole job always shows alone: it gets the whole column. */
  const wholeJobLogHeight = $derived(
    column.height === null
      ? undefined
      : Math.max(MIN_LOG_HEIGHT, column.height - PANEL_HEADER_HEIGHT - PANEL_BORDER_HEIGHT),
  );

  const navButtonClass = (isOpen: boolean) =>
    cn(
      'shrink-0 rounded-lg border border-border px-3 py-2 text-left text-sm transition-colors hover:bg-accent hover:text-accent-foreground',
      isOpen && 'border-primary bg-primary/10 text-primary',
    );
</script>

<!--
  The whole job, or the nodes the reader opened side by side to compare them.
  Each open panel keeps its own stream; closing a panel cancels it.
-->
{#if !canViewLogs}
  <p class="text-sm italic text-muted-foreground">{t(jobsMessages.logsDenied)}</p>
{:else}
  <div class="flex min-h-0 flex-1 flex-col gap-3">
    <div class="flex items-center gap-2">
      <div class="flex size-8 items-center justify-center rounded-lg bg-primary/10">
        <TerminalIcon class="size-4 text-primary" />
      </div>
      <h2 class="text-lg font-semibold text-foreground">{t(jobsMessages.logs)}</h2>
      <span class="flex items-center gap-1.5 text-sm text-muted-foreground">
        <RadioIcon class="size-4 animate-pulse text-green-500" />
        {t(jobsMessages.streamingLive)}
      </span>
    </div>

    <div class="flex min-h-0 flex-1 flex-col gap-3 lg:flex-row">
      <nav
        aria-label={t(jobsMessages.nodeExecutions)}
        class="flex shrink-0 gap-1.5 overflow-x-auto lg:w-60 lg:flex-col lg:overflow-x-visible lg:overflow-y-auto"
      >
        <button
          type="button"
          onclick={onShowWholeJob}
          aria-pressed={isWholeJobOpen}
          class={navButtonClass(isWholeJobOpen)}
        >
          {t(jobsMessages.wholeJob)}
        </button>

        <span aria-hidden="true" class="w-px shrink-0 self-stretch bg-border lg:h-px lg:w-auto"
        ></span>

        {#each nodes as { node, id } (id)}
          {@const config = getStatusConfig(node.state)}
          {@const Icon = getStatusIcon(node.state)}
          {@const duration = calculateExecutionDuration(node.startedAt, node.finishedAt)}
          <button
            type="button"
            onclick={() => onToggleNode(id)}
            aria-pressed={openNodeIds.includes(id)}
            class={cn('flex items-center gap-2', navButtonClass(openNodeIds.includes(id)))}
          >
            <Icon class={cn('size-4 shrink-0', config.iconClassName)} />
            <span class="min-w-0 flex-1 truncate text-sm font-medium">{id}</span>
            <span class="shrink-0 text-xs text-muted-foreground">
              {duration === null ? '-' : formatDuration(duration)}
            </span>
          </button>
        {/each}
      </nav>

      <div
        use:column.measure
        class="flex min-h-0 min-w-0 flex-1 flex-col gap-3 overflow-y-auto"
      >
        {#if isWholeJobOpen}
          <section
            aria-label={t(jobsMessages.wholeJob)}
            class="flex min-w-0 shrink-0 flex-col overflow-hidden rounded-xl border border-border shadow-sm"
          >
            <header
              class="flex h-9 shrink-0 items-center border-b border-border bg-muted/40 px-3"
            >
              <p class="truncate text-sm font-medium text-foreground">
                {t(jobsMessages.wholeJob)}
              </p>
            </header>
            <JobLogDisplay jobId={job.id} maxHeight={wholeJobLogHeight} />
          </section>
        {:else}
          {#each openNodes as { node, id } (id)}
            {@const config = getStatusConfig(node.state)}
            {@const Icon = getStatusIcon(node.state)}
            {@const collapsed = collapsedIds.has(id)}
            <section
              aria-label={id}
              class="flex min-w-0 shrink-0 flex-col overflow-hidden rounded-xl border border-border shadow-sm"
            >
              <header
                class="flex shrink-0 items-center border-b border-border bg-muted/40 pr-2"
              >
                <Button
                  variant="ghost"
                  type="button"
                  onclick={() => toggleCollapse(id)}
                  aria-expanded={!collapsed}
                  aria-label={collapsed
                    ? t(jobsMessages.expandLogsFor(id))
                    : t(jobsMessages.collapseLogsFor(id))}
                  class="h-auto min-w-0 flex-1 justify-start gap-3 rounded-none p-3 hover:scale-100"
                >
                  {#if collapsed}
                    <ChevronRightIcon class="size-4 text-muted-foreground" />
                  {:else}
                    <ChevronDownIcon class="size-4 text-muted-foreground" />
                  {/if}
                  <Icon class={cn('size-5', config.iconClassName)} />
                  <p class="min-w-0 truncate text-sm font-medium text-foreground">{id}</p>
                  <Badge variant="outline" class={config.badgeClassName}>
                    {t(config.label)}
                  </Badge>
                </Button>
                <button
                  type="button"
                  onclick={() => onToggleNode(id)}
                  aria-label={t(jobsMessages.closeLogsFor(id))}
                  class="shrink-0 rounded-md p-1 text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
                >
                  <XIcon class="size-4" />
                </button>
              </header>
              <!-- Collapsing only hides the log: the stream stays open. -->
              <div class={cn(collapsed && 'hidden')}>
                <JobLogDisplay jobId={job.id} nodeId={id} maxHeight={NODE_LOG_HEIGHT} />
              </div>
            </section>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}
