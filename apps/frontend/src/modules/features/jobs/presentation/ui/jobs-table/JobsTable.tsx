import type { JobResponse } from '@/generated/job.ts';
import { useJobsStore } from '@/modules/features/jobs/presentation/stores/useJobsStore.ts';
import { ListCard, type ListCardSection } from '@shared/presentation/ui';
import { JOB_COLUMNS } from '@/modules/features/jobs/presentation/config/jobsTableConfig.ts';
import { JobRow } from '@/modules/features/jobs/presentation/ui/jobs-table/JobRow.tsx';
import { cn } from '@shared/presentation/utils';

type JobsTableProps = {
  jobs: JobResponse[];
  pipelineId: string;
};

const headerSections: ListCardSection[] = JOB_COLUMNS.map(column => ({
  width: column.width,
  className: cn(
    'h-full flex justify-center items-center gap-4 shrink-0 text-slate-500 text-sm',
    column.id === 'actions' && 'flex-1',
  ),
  noSeparator: column.noSeparator,
  content: (
    <span className='w-full h-full rounded-2xl hover:bg-primary-foreground hover:shadow flex items-center justify-center transition-transform hover:scale-110 text-xs font-semibold uppercase tracking-wider'>
      {column.label}
    </span>
  ),
}));

export const JobsTable = ({ jobs, pipelineId }: JobsTableProps) => {
  const selectJob = useJobsStore(state => state.selectJob);
  const selectedJobIds = useJobsStore(state => state.selectedJobIds);

  return (
    <div className={'flex flex-col h-full gap-3'}>
      <ListCard sections={headerSections} className='hover:bg-transparent px-4 py-2 mb-4' />
      <div className='h-full flex flex-col gap-2'>
        {jobs.map((job, index) => (
          <JobRow
            key={index}
            selected={selectedJobIds.includes(job.jobId)}
            onClick={() => {
              selectJob(job.jobId);
            }}
            job={job}
            pipelineId={pipelineId}
          />
        ))}
      </div>
    </div>
  );
};

