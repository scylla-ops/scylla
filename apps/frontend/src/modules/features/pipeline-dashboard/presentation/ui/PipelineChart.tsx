import { cn } from '@shared/presentation/utils';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/modules/shared/presentation/ui/shadcn/tooltip';
import { usePipelineJobs } from '@/modules/shared/presentation/hooks/usePipelineJobs';
import { mapJobStatusToChartStatus, type ChartDataPoint } from '@/modules/shared/utils/jobStatusMapper';
import { calculateDuration, formatDuration, getRelativeTime } from '@/modules/shared/utils/dateUtils';
import { Skeleton } from '@/modules/shared/presentation/ui/shadcn/skeleton';

type PipelineChartProps = {
  pipelineId: string;
  maxJobs?: number;
};

export const PipelineChart = ({ pipelineId, maxJobs = 16 }: PipelineChartProps) => {
  const { jobs, isLoading, isError } = usePipelineJobs(pipelineId, { limit: maxJobs });

  // Transform jobs to chart data points
  const chartData: ChartDataPoint[] = jobs
    .slice(0, maxJobs)
    .map((job, index) => ({
      jobId: job.jobId,
      status: mapJobStatusToChartStatus(job.status),
      createdAt: job.createdAt,
      updatedAt: job.updatedAt,
      duration: calculateDuration(job.createdAt, job.updatedAt),
      runNumber: jobs.length - index,
    }))
    .reverse(); // Reverse to show oldest first (left) to newest (right)

  // Loading state
  if (isLoading) {
    return (
      <div className='w-full flex items-center gap-2 h-10 py-1 overflow-hidden rounded-md px-1'>
        {[...Array(maxJobs)].map((_, index) => (
          <Skeleton key={index} className='flex-1 min-w-[4px] h-full rounded-sm' />
        ))}
      </div>
    );
  }

  // Error state
  if (isError) {
    return (
      <div className='w-full flex items-center justify-center h-10 py-1'>
        <span className='text-xs text-slate-400 italic'>Error loading jobs</span>
      </div>
    );
  }

  // Empty state
  if (chartData.length === 0) {
    return (
      <div className='w-full flex items-center justify-center h-10 py-1'>
        <span className='text-xs text-slate-400 italic'>No jobs yet</span>
      </div>
    );
  }

  return (
    <TooltipProvider delayDuration={100}>
      <div className='w-full flex items-center gap-2 h-10 py-1 overflow-hidden rounded-md px-1'>
        {chartData.map((dataPoint) => {
          const statusLabel =
            dataPoint.status === 1 ? 'Success' : dataPoint.status === 0 ? 'Failed' : 'Running...';

          return (
            <Tooltip key={dataPoint.jobId}>
              <TooltipTrigger asChild>
                <div
                  className={cn(
                    'flex-1 min-w-[4px] h-full rounded-sm transition-all duration-150 cursor-help',

                    // Success
                    dataPoint.status === 1 &&
                      'bg-emerald-400/80 hover:bg-emerald-500 hover:scale-y-110',

                    // Error
                    dataPoint.status === 0 && 'bg-red-400/80 hover:bg-red-500 hover:scale-y-110',

                    // running
                    dataPoint.status === 2 &&
                      'bg-blue-500 animate-[smooth-pulse_2s_infinite] ring-4 ring-blue-400/30 ring-inset hover:scale-y-110',
                  )}
                />
              </TooltipTrigger>
              <TooltipContent side='top' className='text-xs p-3 shadow-lg border-slate-200'>
                <div className='flex flex-col gap-1.5'>
                  <div className='flex items-center justify-between gap-4'>
                    <span className='font-bold text-slate-400'>Run #{dataPoint.runNumber}</span>
                    <span className='text-[10px] text-slate-400 font-mono'>
                      {dataPoint.jobId.slice(0, 8)}...
                    </span>
                  </div>

                  <div className='flex items-center gap-2'>
                    <div
                      className={cn(
                        'w-2 h-2 rounded-full',
                        dataPoint.status === 1 && 'bg-emerald-500',
                        dataPoint.status === 0 && 'bg-red-500',
                        dataPoint.status === 2 && 'bg-blue-500 animate-pulse',
                      )}
                    />
                    <span
                      className={cn(
                        'font-semibold',
                        dataPoint.status === 1 && 'text-emerald-600',
                        dataPoint.status === 0 && 'text-red-600',
                        dataPoint.status === 2 && 'text-blue-600',
                      )}
                    >
                      {statusLabel}
                    </span>
                  </div>

                  <span className='text-[10px] text-slate-500 italic border-t border-slate-100 pt-1 mt-1'>
                    {dataPoint.status === 2
                      ? `Started ${getRelativeTime(dataPoint.createdAt)}`
                      : `Finished ${getRelativeTime(dataPoint.updatedAt)}`}{' '}
                    • Duration: {formatDuration(dataPoint.duration)}
                  </span>
                </div>
              </TooltipContent>
            </Tooltip>
          );
        })}
      </div>
    </TooltipProvider>
  );
};
