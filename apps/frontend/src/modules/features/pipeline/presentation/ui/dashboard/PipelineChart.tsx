import { cn } from '@shared/presentation/utils';
import { Skeleton } from '@shadcn/skeleton.tsx';
import { StatusBar, type StatusBarItem } from '@shared/presentation/ui/StatusBar.tsx';
import { getStatusConfig } from '@shared/utils/status-config.ts';
import { calculateDuration, formatDuration, getRelativeTime } from '@shared/utils/date-utils.ts';
import type { Job } from '@/modules/features/jobs/domain/models/job.model.ts';

type PipelineChartProps = {
  maxJobs?: number;
  jobs: Job[];
  isLoading?: boolean;
  isError?: boolean;
};

export const PipelineChart = ({ jobs, isLoading, isError, maxJobs }: PipelineChartProps) => {
  if (isLoading) {
    return (
      <div className='w-full flex items-center gap-2 h-10 py-1 overflow-hidden rounded-md px-1'>
        {[...Array(maxJobs)].map((_, index) => (
          <Skeleton key={index} className='flex-1 min-w-[4px] h-full rounded-sm' />
        ))}
      </div>
    );
  }

  if (isError) {
    return (
      <div className='w-full flex items-center justify-center h-10 py-1'>
        <span className='text-xs text-slate-400 italic'>Error loading jobs</span>
      </div>
    );
  }

  const items: StatusBarItem[] = jobs
    .slice(0, maxJobs)
    .map((job, index) => {
      const config = getStatusConfig(job.status);
      const duration = calculateDuration(job.createdAt, job.updatedAt);
      const runNumber = jobs.length - index;

      return {
        id: job.id,
        status: job.status,
        tooltip: (
          <div className='flex flex-col gap-1.5'>
            <div className='flex items-center justify-between gap-4'>
              <span className='font-bold text-slate-400'>Run #{runNumber}</span>
              <span className='text-[10px] text-slate-400 font-mono'>{job.id.slice(0, 8)}...</span>
            </div>
            <div className='flex items-center gap-2'>
              <div className={cn('w-2 h-2 rounded-full', config.dotClassName)} />
              <span className={cn('font-semibold', config.textClassName)}>{config.label}</span>
            </div>
            <span className='text-[10px] text-slate-500 italic border-t border-slate-100 pt-1 mt-1'>
              {job.status === 'running' || job.status === 'pending'
                ? `Started ${getRelativeTime(job.createdAt)}`
                : `Finished ${getRelativeTime(job.updatedAt)}`}{' '}
              • Duration: {formatDuration(duration)}
            </span>
          </div>
        ),
      };
    })
    .reverse();

  return <StatusBar items={items} emptyLabel='No jobs yet' height='h-10' className='px-1' />;
};
