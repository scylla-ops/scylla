import { Clock } from 'lucide-react';
import {
  calculateExecutionDuration,
  formatDuration,
  getRelativeTime,
} from '@shared/utils/date-utils.ts';
import { useNow } from '@shared/presentation/hooks/use-now.ts';
import type { JobEntity } from '@/modules/features/jobs';
import { Trans, useLingui } from '@lingui/react/macro';

type PipelineMetadataProps = {
  jobs: JobEntity[];
  /** Makes the last run open its job details page. */
  onSelectJob?: (jobId: string) => void;
};

/**
 * Displays the metadata of the last job for a pipeline
 */
export const PipelineLastJob = ({ jobs, onSelectJob }: PipelineMetadataProps) => {
  const { t } = useLingui();
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
        <span className='text-xs italic truncate'>
          <Trans>No jobs yet</Trans>
        </span>
      </div>
    );
  }

  const duration = calculateExecutionDuration(lastJob.startedAt, lastJob.finishedAt);
  const lastRun = getRelativeTime(lastJob.updatedAt);

  const content = (
    <>
      <div className='flex flex-row w-full items-center justify-center gap-1.5'>
        <Clock className='w-3.5 h-3.5' />
        <span>{duration === null ? '-' : formatDuration(duration)}</span>
      </div>
      <span className='text-xs italic truncate'>{lastRun}</span>
    </>
  );

  const className = 'flex flex-col w-full items-center justify-center gap-1';

  if (!onSelectJob) return <div className={className}>{content}</div>;

  return (
    <button
      type='button'
      aria-label={t`Open the last run`}
      onClick={event => {
        event.stopPropagation();
        onSelectJob(lastJob.id);
      }}
      className={`${className} rounded-md px-1 py-0.5 cursor-pointer transition-colors hover:bg-accent hover:text-accent-foreground`}
    >
      {content}
    </button>
  );
};
