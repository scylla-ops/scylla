import type { JobNodeExecution } from '@/modules/features/jobs/domain/structs/job.struct.ts';
import { StatusBar, type StatusBarItem } from '@shared/presentation/ui/data-display/StatusBar.tsx';
import { useMemo } from 'react';
import { getStatusConfig } from '@shared/utils/status-config.ts';
import { cn } from '@shared/presentation/utils';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@shadcn/tooltip.tsx';
import { Trans, useLingui } from '@lingui/react/macro';
import { formatTime, calculateExecutionDuration, formatDuration } from '@shared/utils/date-utils.ts';

type JobTimelineProps = {
  nodeExecutions: JobNodeExecution[];
  /**
   * Makes the segments activatable. The detailed view targets the node that was
   * clicked; the grouped view stands for several nodes at once, so it passes
   * none and the caller falls back to the job itself.
   */
  onSelectNode?: (nodeId?: string) => void;
};

/** Threshold above which nodes are grouped by status */
const COLLAPSE_THRESHOLD = 10;

interface StatusGroup {
  status: string;
  count: number;
  nodes: JobNodeExecution[];
}

/**
 * Display a timeline bar showing the execution state of each node.
 * When there are many nodes, they are grouped by status into proportional segments.
 */
export const JobTimeline = ({ nodeExecutions, onSelectNode }: JobTimelineProps) => {
  const { i18n, t } = useLingui();
  const shouldCollapse = nodeExecutions.length > COLLAPSE_THRESHOLD;

  // Grouped view for large pipelines
  const groups = useMemo<StatusGroup[]>(() => {
    if (!shouldCollapse) return [];
    const map = new Map<string, JobNodeExecution[]>();
    for (const node of nodeExecutions) {
      const key = node.state;
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(node);
    }
    return Array.from(map.entries()).map(([status, nodes]) => ({
      status,
      count: nodes.length,
      nodes,
    }));
  }, [nodeExecutions, shouldCollapse]);

  if (nodeExecutions.length === 0) {
    return <StatusBar items={[]} emptyLabel={<Trans>No nodes</Trans>} />;
  }

  // Detailed view for small pipelines
  if (!shouldCollapse) {
    const items: StatusBarItem[] = nodeExecutions.map((node, index) => {
      const duration = calculateExecutionDuration(node.startedAt, node.finishedAt);
      const nodeId = node.id || String(index);
      return {
        id: nodeId,
        status: node.state,
        onSelect: onSelectNode ? () => onSelectNode(nodeId) : undefined,
        label: t`Node ${nodeId}`,
        tooltip: (
          <div className='text-xs'>
            <p className='font-semibold'>{node.id}</p>
            <p>
              <Trans>State: {i18n._(getStatusConfig(node.state).label)}</Trans>
            </p>
            {node.startedAt && (
              <p>
                <Trans>Started: {formatTime(node.startedAt)}</Trans>
              </p>
            )}
            {node.finishedAt && (
              <p>
                <Trans>Finished: {formatTime(node.finishedAt)}</Trans>
              </p>
            )}
            {duration !== null && (
              <p>
                <Trans>Duration: {formatDuration(duration)}</Trans>
              </p>
            )}
          </div>
        ),
      };
    });
    return <StatusBar items={items} emptyLabel={<Trans>No nodes</Trans>} />;
  }

  // Collapsed proportional view
  const total = nodeExecutions.length;

  return (
    <TooltipProvider delayDuration={100}>
      <div className='w-full flex items-center gap-0.5 py-1 h-6 overflow-hidden rounded-md'>
        {groups.map(group => {
          const config = getStatusConfig(group.status);
          const pct = (group.count / total) * 100;
          const groupClassName = cn(
            'h-full rounded-sm transition-all duration-150 relative flex items-center justify-center',
            config.barClassName,
            config.barHoverClassName,
            onSelectNode && 'cursor-pointer',
          );
          const count = pct > 8 && (
            <span className='text-[10px] font-semibold text-primary-foreground drop-shadow-sm select-none'>
              {group.count}
            </span>
          );

          return (
            <Tooltip key={group.status}>
              <TooltipTrigger asChild>
                {onSelectNode ? (
                  <button
                    type='button'
                    aria-label={t`${group.count} ${i18n._(config.label)} nodes`}
                    onClick={event => {
                      event.stopPropagation();
                      onSelectNode();
                    }}
                    className={groupClassName}
                    style={{ width: `${pct}%`, minWidth: 18 }}
                  >
                    {count}
                  </button>
                ) : (
                  <div className={groupClassName} style={{ width: `${pct}%`, minWidth: 18 }}>
                    {count}
                  </div>
                )}
              </TooltipTrigger>
              <TooltipContent side='top' className='text-xs p-3 shadow-lg'>
                <div className='space-y-1'>
                  <p className='font-semibold capitalize'>{i18n._(config.label)}</p>
                  <p>
                    <Trans>
                      {group.count} / {total} nodes ({Math.round(pct)}%)
                    </Trans>
                  </p>
                  {group.count <= 8 && (
                    <ul className='mt-1 space-y-0.5 text-muted-foreground'>
                      {group.nodes.map(n => (
                        <li key={n.id}>• {n.id}</li>
                      ))}
                    </ul>
                  )}
                </div>
              </TooltipContent>
            </Tooltip>
          );
        })}
      </div>
    </TooltipProvider>
  );
};
