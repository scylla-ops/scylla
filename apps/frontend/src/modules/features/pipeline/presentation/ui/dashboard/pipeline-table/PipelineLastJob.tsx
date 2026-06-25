import { Clock } from 'lucide-react';
import {
  calculateExecutionDuration,
  formatDuration,
  getRelativeTime,
} from '@shared/utils/date-utils.ts';
import { useNow } from '@shared/presentation/hooks/use-now.ts';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';

type PipelineMetadataProps = {
  jobs: JobEntity[];
};

/**
 * Displays the metadata of the last job for a pipeline
 */
export const PipelineLastJob = ({ jobs }: PipelineMetadataProps) => {
  const lastJob = jobs[0];
  const isLive = lastJob?.status === 'running' || lastJob?.status === 'pending';
  useNow(isLive);

  if (!lastJob) {
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

  const duration = calculateExecutionDuration(lastJob.startedAt, lastJob.finishedAt);
  const lastRun = getRelativeTime(lastJob.updatedAt);

  return (
    <div className={'flex flex-col w-full items-center justify-center gap-1'}>
      <div className='flex flex-row w-full items-center justify-center gap-1.5'>
        <Clock className='w-3.5 h-3.5' />
        <span>{duration === null ? '-' : formatDuration(duration)}</span>
      </div>
      <span className='text-xs italic truncate'>{lastRun}</span>
    </div>
  );
};
