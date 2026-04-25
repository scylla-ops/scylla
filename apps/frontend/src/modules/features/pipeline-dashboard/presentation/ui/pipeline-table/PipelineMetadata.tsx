import { Clock } from 'lucide-react';
import type { JobResponse } from '@/generated/job.ts';
import {
  calculateDuration,
  formatDuration,
  getRelativeTime,
} from '@/modules/shared/utils/dateUtils';

type PipelineMetadataProps = {
  jobs: JobResponse[];
};

/**
 * Displays the metadata of the last job for a pipeline
 */
export const PipelineMetadata = ({ jobs }: PipelineMetadataProps) => {
  if (jobs.length === 0) {
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
