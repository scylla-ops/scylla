import { usePipelinesJobs } from '@/modules/features/jobs/presentation/hooks/use-pipelines-jobs.ts';
import { JobsHeader } from '@/modules/features/jobs/presentation/ui/JobsHeader.tsx';
import { useParams, useNavigate } from 'react-router-dom';
import { JobsTable } from '@/modules/features/jobs/presentation/ui/jobs-table';
import { ErrorState } from '@/modules/shared/presentation/ui/ErrorState.tsx';
import { Trans } from '@lingui/react/macro';
import { Pagination } from '@shared/presentation/ui/Pagination.tsx';

export const JobsPage = () => {
  const { pipelineId } = useParams<{ pipelineId: string }>();
  const navigate = useNavigate();
  const { isLoading, jobs, isError, errorMessage, refetch, paginationInfo, setPage } =
    usePipelinesJobs(pipelineId || '');

  const handleBack = () => {
    navigate(-1);
  };

  if (!pipelineId) {
    return <ErrorState message='Pipeline ID is missing' />;
  }

  if (isLoading || !jobs) {
    return <></>;
  }

  if (isError) {
    return <ErrorState message={String(errorMessage) || 'Unable to load jobs'} />;
  }

  return (
    <div className='flex flex-col gap-4 w-full h-full'>
      <JobsHeader
        numberOfJobs={paginationInfo?.totalCount ?? jobs.length}
        pipelineId={pipelineId}
        onRefresh={() => refetch()}
        onBack={handleBack}
      />
      <div className='flex-1 min-h-0 overflow-auto'>
        <div className={'relative'}>
          {jobs.length > 0 ? (
            <JobsTable jobs={jobs} pipelineId={pipelineId} />
          ) : (
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

      {paginationInfo && paginationInfo.totalPages > 1 && (
        <div className='shrink-0 pt-2'>
          <Pagination paginationInfo={paginationInfo} onPageChange={setPage} />
        </div>
      )}
    </div>
  );
};
