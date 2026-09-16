import { usePipelinesJobs } from '@/modules/features/jobs/presentation/hooks/use-pipelines-jobs.ts';
import { JobsHeader } from '@/modules/features/jobs/presentation/ui/JobsHeader.tsx';
import { useParams } from 'react-router-dom';
import { JobsTable } from '@/modules/features/jobs/presentation/ui/jobs-table';
import { ErrorState } from '@/modules/shared/presentation/ui/feedback/ErrorState.tsx';
import { Trans } from '@lingui/react/macro';
import { PaginationSlot } from '@shared/presentation/ui/data-display/Pagination.tsx';
import { NoAgentsBanner } from '@/modules/features/agents';

interface JobsPageProps {
  /** Passed through to {@link JobsHeader} — see the note on its `onRun` prop. */
  onRun?: () => Promise<void>;
}

export const JobsPage = ({ onRun }: JobsPageProps) => {
  const { pipelineId } = useParams<{ pipelineId: string }>();
  const { jobs, isError, errorMessage, refetch, paginationInfo, setPage, containerRef } =
    usePipelinesJobs(pipelineId || '');

  if (!pipelineId) {
    return <ErrorState message={<Trans>Pipeline ID is missing</Trans>} />;
  }

  if (isError) {
    return <ErrorState message={String(errorMessage) || 'Unable to load jobs'} />;
  }

  // The frame renders before the jobs do: the table area has to be in the DOM
  // for its height to be measured, and that height decides what to fetch.
  return (
    <div className='flex flex-col gap-4 w-full h-full min-h-0'>
      <JobsHeader
        numberOfJobs={paginationInfo?.totalCount ?? jobs?.length ?? 0}
        jobIds={jobs?.map(job => job.id) ?? []}
        pipelineId={pipelineId}
        onRefresh={() => refetch()}
        onRun={onRun}
      />
      <NoAgentsBanner hasPendingJobs={jobs?.some(job => job.status === 'pending') ?? false} />
      <div ref={containerRef} className='flex-1 min-h-0 overflow-auto'>
        <div className={'relative'}>
          {jobs && jobs.length > 0 && <JobsTable jobs={jobs} pipelineId={pipelineId} />}
          {jobs && jobs.length === 0 && (
            <div className='flex items-center justify-center h-full min-h-[400px]'>
              <div className='text-center space-y-2'>
                <p className='text-muted-foreground'>
                  <Trans>No jobs found</Trans>
                </p>
                <p className='text-sm text-muted-foreground'>
                  <Trans>Run your pipeline to create the first job</Trans>
                </p>
              </div>
            </div>
          )}
        </div>
      </div>

      <PaginationSlot paginationInfo={paginationInfo} onPageChange={setPage} />
    </div>
  );
};
