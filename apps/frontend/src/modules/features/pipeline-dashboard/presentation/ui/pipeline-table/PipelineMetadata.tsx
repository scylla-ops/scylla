import { Clock } from 'lucide-react';
import { usePipelineJobs } from '@/modules/shared/presentation/hooks/usePipelineJobs';
import {
  calculateDuration,
  formatDuration,
  getRelativeTime,
} from '@/modules/shared/utils/dateUtils';
import { Skeleton } from '@/modules/shared/presentation/ui/shadcn/skeleton';

type PipelineMetadataProps = {
  pipelineId: string;
};

/**
 * Displays the metadata of the last job for a pipeline
 */
export const PipelineMetadata = ({ pipelineId }: PipelineMetadataProps) => {
  const { jobs, isLoading, isError } = usePipelineJobs(pipelineId, { limit: 1 });

  // Loading state
  if (isLoading) {
    return (
      <>
        <Skeleton className='w-16 h-4' />
        <Skeleton className='w-16 h-3' />
      </>
    );
  }

  // Error or empty state
  if (isError || jobs.length === 0) {
    return (
      <div className={'flex flex-col w-full items-center justify-center gap-1'}>
        <div className='flex items-center justify-center gap-1.5'>
          <Clock className='w-3.5 h-3.5' />
          <span>-</span>
        </div>
        <span className='text-xs italic truncate'>No jobs yet</span>
      </div>
    );
  }

  const lastJob = jobs[0];
  const duration = calculateDuration(lastJob.createdAt, lastJob.updatedAt);
  const lastRun = getRelativeTime(lastJob.updatedAt);

  return (
    <div className={'flex flex-col w-full items-center justify-center gap-1'}>
      <div className='flex flex-row w-full items-center justify-center gap-1.5'>
        <Clock className='w-3.5 h-3.5' />
        <span>{formatDuration(duration)}</span>
      </div>
      <span className='text-xs italic truncate'>{lastRun}</span>
    </div>
  );
};
