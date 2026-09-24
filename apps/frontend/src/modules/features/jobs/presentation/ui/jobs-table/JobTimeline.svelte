<script lang="ts">
  import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn';
  import { StatusBar, type StatusBarItem } from '@shared/presentation/ui';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import {
    calculateExecutionDuration,
    formatDuration,
    formatTime,
  } from '@shared/utils/date-utils.ts';
  import { getStatusConfig } from '@shared/utils/status-config.ts';
  import type { JobNodeExecution } from '../../../domain/structs/job.struct.ts';
  import { jobsMessages } from '../jobs.messages.ts';
  import { groupByStatus, nodeIdOf, shouldCollapse } from './job-timeline.calculator.ts';

  interface Props {
    nodeExecutions: JobNodeExecution[];
    /**
     * Makes the segments activatable. The detailed view targets the node that
     * was clicked; the grouped view stands for several nodes at once, so it
     * passes none and the caller falls back to the job itself.
     */
    onSelectNode?: (nodeId?: string) => void;
  }

  let { nodeExecutions, onSelectNode }: Props = $props();

  /** A bar segment, plus the node it stands for — what the tooltip reads. */
  interface NodeItem extends StatusBarItem {
    node: JobNodeExecution;
  }

  const collapsed = $derived(shouldCollapse(nodeExecutions.length));

  const items = $derived.by((): NodeItem[] =>
    collapsed
      ? []
      : nodeExecutions.map((node, index) => {
          const nodeId = nodeIdOf(node, index);
          return {
            id: nodeId,
            status: node.state,
            node,
            label: t(jobsMessages.nodeLabel(nodeId)),
            onSelect: onSelectNode ? () => onSelectNode(nodeId) : undefined,
          };
        }),
  );

  const groups = $derived(collapsed ? groupByStatus(nodeExecutions) : []);
  const total = $derived(nodeExecutions.length);
</script>

<!--
  A job's node executions as one bar: a segment per node, or — past the
  threshold, where per-node segments stop being readable — one proportional
  segment per status.
-->
{#snippet nodeTooltip(item: NodeItem)}
  {@const duration = calculateExecutionDuration(item.node.startedAt, item.node.finishedAt)}
  <div class="text-xs">
    <p class="font-semibold">{item.node.id}</p>
    <p>{t(jobsMessages.nodeState(t(getStatusConfig(item.node.state).label)))}</p>
    {#if item.node.startedAt}
      <p>{t(jobsMessages.nodeStarted(formatTime(item.node.startedAt)))}</p>
    {/if}
    {#if item.node.finishedAt}
      <p>{t(jobsMessages.nodeFinished(formatTime(item.node.finishedAt)))}</p>
    {/if}
    {#if duration !== null}
      <p>{t(jobsMessages.nodeDuration(formatDuration(duration)))}</p>
    {/if}
  </div>
{/snippet}

{#if !collapsed}
  <StatusBar {items} tooltip={nodeTooltip} emptyLabel={t(jobsMessages.noNodes)} />
{:else}
  <div class="w-full flex items-center gap-0.5 py-1 h-6 overflow-hidden rounded-md">
    {#each groups as group (group.status)}
      {@const config = getStatusConfig(group.status)}
      {@const groupClass = cn(
        'h-full rounded-sm transition-all duration-150 relative flex items-center justify-center',
        config.barClassName,
        config.barHoverClassName,
        onSelectNode && 'cursor-pointer',
      )}
      <Tooltip delayDuration={100}>
        <TooltipTrigger>
          {#snippet child({ props })}
            {#if onSelectNode}
              <button
                type="button"
                aria-label={t(jobsMessages.groupLabel(group.count, t(config.label)))}
                {...props}
                onclick={event => {
                  (props.onclick as ((e: MouseEvent) => void) | undefined)?.(event);
                  event.stopPropagation();
                  // A grouped segment stands for several nodes and names none,
                  // which comes back to the job as a whole.
                  onSelectNode();
                }}
                class={groupClass}
                style="width: {group.percent}%; min-width: 18px"
              >
                {#if group.percent > 8}
                  <span
                    class="text-[10px] font-semibold text-primary-foreground drop-shadow-sm select-none"
                  >
                    {group.count}
                  </span>
                {/if}
              </button>
            {:else}
              <div
                {...props}
                class={groupClass}
                style="width: {group.percent}%; min-width: 18px"
              >
                {#if group.percent > 8}
                  <span
                    class="text-[10px] font-semibold text-primary-foreground drop-shadow-sm select-none"
                  >
                    {group.count}
                  </span>
                {/if}
              </div>
            {/if}
          {/snippet}
        </TooltipTrigger>
        <TooltipContent side="top" class="text-xs p-3 shadow-lg">
          <div class="space-y-1">
            <p class="font-semibold capitalize">{t(config.label)}</p>
            <p>
              {t(jobsMessages.groupShare(group.count, total, Math.round(group.percent)))}
            </p>
            {#if group.count <= 8}
              <ul class="mt-1 space-y-0.5 text-muted-foreground">
                {#each group.nodes as node (node.id)}
                  <li>• {node.id}</li>
                {/each}
              </ul>
            {/if}
          </div>
        </TooltipContent>
      </Tooltip>
    {/each}
  </div>
{/if}
