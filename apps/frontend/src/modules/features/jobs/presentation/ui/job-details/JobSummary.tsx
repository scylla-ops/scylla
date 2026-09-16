import type { ReactNode } from 'react';
import { Trans } from '@lingui/react/macro';
import { Badge } from '@shadcn';
import { CopyableText } from '@shared/presentation/ui/data-display/CopyableText.tsx';
import { useNow } from '@shared/presentation/hooks/use-now.ts';
import { calculateExecutionDuration, formatDate, formatDuration } from '@shared/utils/date-utils.ts';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import { JobStatus } from '@/modules/features/jobs/presentation/ui/jobs-table/JobStatus.tsx';
import { JobTimeline } from '@/modules/features/jobs/presentation/ui/jobs-table/JobTimeline.tsx';

interface JobSummaryProps {
  job: JobEntity;
  onSelectNode: (nodeId?: string) => void;
}

const Fact = ({ label, children }: { label: ReactNode; children: ReactNode }) => (
  <span className='flex items-center gap-1.5'>
    <span className='font-mono text-[10px] uppercase tracking-wide text-muted-foreground'>
      {label}
    </span>
    <span className='font-mono text-xs text-foreground'>{children}</span>
  </span>
);

export const JobSummary = ({ job, onSelectNode }: JobSummaryProps) => {
  const isLive = job.status === 'running' || job.status === 'pending';
  useNow(isLive);

  const duration = calculateExecutionDuration(job.startedAt, job.finishedAt);

  return (
    <div className='flex flex-col gap-4'>
      <div className='flex flex-wrap items-center justify-between gap-3'>
        <div className='flex min-w-0 items-center gap-3'>
          <h1 className='text-xl font-semibold text-foreground'>
            <Trans>Job</Trans>
          </h1>
          <Badge variant='outline' className='gap-1 pr-1 font-mono text-xs'>
            <CopyableText value={job.id} copyLabel={<Trans>Copy job id</Trans>} />
          </Badge>
        </div>
        <div className='w-fit'>
          <JobStatus job={job} />
        </div>
      </div>

      <div className='flex flex-wrap items-center gap-x-3 gap-y-1.5 rounded-md border border-border bg-card px-3.5 py-2'>
        <Fact label={<Trans>Duration</Trans>}>
          {duration === null ? '-' : formatDuration(duration)}
        </Fact>
        <span className='text-muted-foreground/50'>·</span>
        <Fact label={<Trans context='date-prefix'>Started</Trans>}>
          {job.startedAt ? formatDate(job.startedAt) : '-'}
        </Fact>
        <span className='text-muted-foreground/50'>·</span>
        <Fact label={<Trans context='date-prefix'>Finished</Trans>}>
          {job.finishedAt ? formatDate(job.finishedAt) : '-'}
        </Fact>
        <span className='text-muted-foreground/50'>·</span>
        <Fact label={<Trans context='date-prefix'>Created</Trans>}>{formatDate(job.createdAt)}</Fact>
      </div>

      <div>
        <h2 className='mb-2 text-lg font-semibold text-foreground'>
          <Trans>Timeline</Trans>
        </h2>
        <JobTimeline nodeExecutions={job.nodeExecutions} onSelectNode={onSelectNode} />
      </div>
    </div>
  );
};
