import type { JobNodeResponse } from '@/generated/job.ts';
import { StatusBar, type StatusBarItem } from '@shared/presentation/ui/StatusBar.tsx';

type JobTimelineProps = {
  nodeExecutions: JobNodeResponse[];
};

/**
 * Display a timeline bar showing the execution state of each node
 */
export const JobTimeline = ({ nodeExecutions }: JobTimelineProps) => {
  const items: StatusBarItem[] = nodeExecutions.map((node, index) => ({
    id: node.nodeId || String(index),
    status: node.state,
    tooltip: (
      <div className='text-xs'>
        <p className='font-semibold'>{node.nodeId}</p>
        <p>State: {node.state}</p>
        {node.startedAt && <p>Started: {new Date(node.startedAt).toLocaleTimeString()}</p>}
        {node.finishedAt && <p>Finished: {new Date(node.finishedAt).toLocaleTimeString()}</p>}
      </div>
    ),
  }));

  return <StatusBar items={items} emptyLabel='No nodes' />;
};
