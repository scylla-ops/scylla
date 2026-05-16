import type { JobNodeExecution } from '@/modules/features/jobs/domain/models/job.model.ts';
import { StatusBar, type StatusBarItem } from '@shared/presentation/ui/StatusBar.tsx';
import { useMemo } from 'react';
import { getStatusConfig } from '@shared/utils/status-config.ts';
import { cn } from '@shared/presentation/utils';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@shadcn/tooltip.tsx';

type JobTimelineProps = {
  nodeExecutions: JobNodeExecution[];
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
export const JobTimeline = ({ nodeExecutions }: JobTimelineProps) => {
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
    return <StatusBar items={[]} emptyLabel='No nodes' />;
  }

  // Detailed view for small pipelines
  if (!shouldCollapse) {
    const items: StatusBarItem[] = nodeExecutions.map((node, index) => ({
      id: node.id || String(index),
      status: node.state,
      tooltip: (
        <div className='text-xs'>
          <p className='font-semibold'>{node.id}</p>
          <p>State: {node.state}</p>
          {node.startedAt && <p>Started: {new Date(node.startedAt).toLocaleTimeString()}</p>}
          {node.finishedAt && <p>Finished: {new Date(node.finishedAt).toLocaleTimeString()}</p>}
        </div>
      ),
    }));
    return <StatusBar items={items} emptyLabel='No nodes' />;
  }

  // Collapsed proportional view
  const total = nodeExecutions.length;

  return (
    <TooltipProvider delayDuration={100}>
      <div className='w-full flex items-center gap-0.5 py-1 h-6 overflow-hidden rounded-md'>
        {groups.map(group => {
          const config = getStatusConfig(group.status);
          const pct = (group.count / total) * 100;

          return (
            <Tooltip key={group.status}>
              <TooltipTrigger asChild>
                <div
                  className={cn(
                    'h-full rounded-sm transition-all duration-150 relative flex items-center justify-center',
                    config.barClassName,
                    config.barHoverClassName,
                  )}
                  style={{ width: `${pct}%`, minWidth: 18 }}
                >
                  {pct > 8 && (
                    <span className='text-[10px] font-semibold text-white drop-shadow-sm select-none'>
                      {group.count}
                    </span>
                  )}
                </div>
              </TooltipTrigger>
              <TooltipContent side='top' className='text-xs p-3 shadow-lg border-slate-200'>
                <div className='space-y-1'>
                  <p className='font-semibold capitalize'>{config.label}</p>
                  <p>
                    {group.count} / {total} nodes ({Math.round(pct)}%)
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
