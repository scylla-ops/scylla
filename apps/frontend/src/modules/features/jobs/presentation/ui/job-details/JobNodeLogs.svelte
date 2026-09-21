<script lang="ts">
  import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
  import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
  import RadioIcon from '@lucide/svelte/icons/radio';
  import TerminalIcon from '@lucide/svelte/icons/terminal';
  import XIcon from '@lucide/svelte/icons/x';
  import { Permission, can } from '@platform/authz';
  import { Badge, Button } from '@shadcn-svelte';
  import { getStatusIcon } from '@shared/presentation/ui-svelte';
  import { createMeasuredHeight } from '@shared/presentation/state/measured-height.svelte.ts';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { calculateExecutionDuration, formatDuration } from '@shared/utils/date-utils.ts';
  import { getStatusConfig } from '@shared/utils/status-config.ts';
  import type { JobEntity } from '../../../domain/entities/job.entity.ts';
  import { nodeIdOf } from '../jobs-table/job-timeline.calculator.ts';
  import JobLogDisplay from '../jobs-log/JobLogDisplay.svelte';
  import { jobsMessages } from '../jobs.messages.ts';

  /** The whole job's `h-9` header, the only one whose height the column must allow for. */
  const PANEL_HEADER_HEIGHT = 36;
  /** The panel's own `border` (1px top + 1px bottom), on top of the header. */
  const PANEL_BORDER_HEIGHT = 2;
  /** Below this a log is a peephole; the column scrolls rather than shrink past it. */
  const MIN_LOG_HEIGHT = 192;
  /** What a node's log stands at, however many are open — the column takes the overflow. */
  const NODE_LOG_HEIGHT = 448;

  interface Props {
    job: JobEntity;
    /** From the URL, in execution order. Ids no node matches are already dropped. */
    openNodeIds: readonly string[];
    isWholeJobOpen: boolean;
    /** Adds or removes one node's panel, leaving the other open ones alone. */
    onToggleNode: (nodeId: string) => void;
    /** Drops every node panel, which is what brings the whole job back. */
    onShowWholeJob: () => void;
  }

  let { job, openNodeIds, isWholeJobOpen, onToggleNode, onShowWholeJob }: Props = $props();

  const canViewLogs = $derived(can(Permission.READ_JOB_LOGS));

  // The column's height comes from the layout and never from the panels inside
  // it, or measuring it would resize what it measures.
  const column = createMeasuredHeight();

  let collapsedIds = $state(new Set<string>());

  const toggleCollapse = (nodeId: string) => {
    const next = new Set(collapsedIds);
    if (next.has(nodeId)) next.delete(nodeId);
    else next.add(nodeId);
    collapsedIds = next;
  };

  const nodes = $derived(
    job.nodeExecutions.map((node, index) => ({ node, id: nodeIdOf(node, index) })),
  );
  const openNodes = $derived(nodes.filter(({ id }) => openNodeIds.includes(id)));

  /** The whole job is only ever shown alone, so its log gets the column entire. */
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
  The job's logs: the job as a whole, or the nodes the reader picked out of it.

  Comparing what two nodes printed is the point, so the nav adds and removes
  node panels rather than switching between them, and each keeps the same
  readable height whether it is alone or one of five — past the room the page
  has, the column scrolls. The whole job is what shows when no node is picked,
  never a panel alongside them, which is why it has no close button: closing the
  last node is what comes back to it.

  Every open panel keeps its own live stream, and closing one unmounts it, which
  is what cancels that stream.
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
              <!--
                The whole line collapses the panel, the way the node list this
                page replaced read: a chevron leading, then the status, the node
                and what it ended as. Only the close button is left out of it —
                a button inside a button is no HTML, and closing is not
                collapsing.
              -->
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
              <!-- Collapsing only hides the log, so its stream stays open — no
                   reconnect when the reader expands it again. -->
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
