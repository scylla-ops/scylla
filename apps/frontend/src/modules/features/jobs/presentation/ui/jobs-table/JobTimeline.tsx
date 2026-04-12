import type { JobNodeResponse } from '@/generated/job.ts';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/modules/shared/presentation/ui/shadcn/tooltip.tsx';

type JobTimelineProps = {
  nodeExecutions: JobNodeResponse[];
};

const stateColors: Record<string, string> = {
  pending: 'bg-slate-300',
  running: 'bg-blue-500 animate-pulse',
  success: 'bg-green-500',
  failed: 'bg-red-500',
};

/**
 * Display a timeline bar showing the execution state of each node
 */
export const JobTimeline = ({ nodeExecutions }: JobTimelineProps) => {
  if (nodeExecutions.length === 0) {
    return (
      <div className='w-full h-6 bg-slate-200 rounded-full flex items-center justify-center'>
        <span className='text-xs text-slate-500'>No nodes</span>
      </div>
    );
  }

  return (
    <div className='w-full flex gap-1'>
      <TooltipProvider>
        {nodeExecutions.map((node, index) => (
          <Tooltip key={index}>
            <TooltipTrigger asChild>
              <div
                className={`h-6 rounded flex-1 transition-all ${stateColors[node.state] || stateColors.pending}`}
              />
            </TooltipTrigger>
            <TooltipContent>
              <div className='text-xs'>
                <p className='font-semibold'>{node.nodeId}</p>
                <p>State: {node.state}</p>
                {node.startedAt && <p>Started: {new Date(node.startedAt).toLocaleTimeString()}</p>}
                {node.finishedAt && (
                  <p>Finished: {new Date(node.finishedAt).toLocaleTimeString()}</p>
                )}
              </div>
            </TooltipContent>
          </Tooltip>
        ))}
      </TooltipProvider>
    </div>
  );
};

