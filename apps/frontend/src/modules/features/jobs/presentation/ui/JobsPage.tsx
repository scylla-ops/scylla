import { usePipelineJobs } from '@/modules/features/jobs/presentation/hooks/usePipelineJobs.ts';
import { Skeleton } from '@/modules/shared/presentation/ui/shadcn/skeleton.tsx';
import { useDelayedLoading } from '@/modules/shared/presentation/hooks/useDelayedLoading.ts';
import { JobsHeader } from '@/modules/features/jobs/presentation/ui/JobsHeader.tsx';
import { useParams, useNavigate } from 'react-router-dom';
import JobsTableSkeleton from '@/modules/features/jobs/presentation/ui/jobs-table/JobsTableSkeleton.tsx';
import { JobsTable } from '@/modules/features/jobs/presentation/ui/jobs-table';

export const JobsPage = () => {
  const { pipelineId } = useParams<{ pipelineId: string }>();
  const navigate = useNavigate();
  const { isLoading, jobs, isError, errorMessage, refetch } = usePipelineJobs(pipelineId || '');
  const showSkeleton = useDelayedLoading(400);

  const handleBack = () => {
    navigate(-1);
  };

  if (!pipelineId) {
    return (
      <div className='flex items-center justify-center h-full'>
        <div className='text-center space-y-2'>
          <p className='text-destructive text-lg font-semibold'>Error</p>
          <p className='text-muted-foreground text-sm'>Pipeline ID is missing</p>
        </div>
      </div>
    );
  }

  if (isLoading && !showSkeleton) {
    return <></>;
  }

  if ((isLoading && showSkeleton) || !jobs) {
    return (
      <div className='flex flex-col gap-4 w-full h-full p-2'>
        <div className='flex items-baseline gap-2'>
          <Skeleton className='h-9 w-56' />
          <Skeleton className='h-5 w-16' />
        </div>
        <div className='h-full flex flex-col gap-2'>
          <JobsTableSkeleton />
        </div>
      </div>
    );
  }

  if (isError) {
    return (
      <div className='flex items-center justify-center h-full'>
        <div className='text-center space-y-2'>
          <p className='text-destructive text-lg font-semibold'>Erreur</p>
          <p className='text-muted-foreground text-sm'>
            {String(errorMessage) || 'Impossible de charger les jobs'}
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className='flex flex-col gap-4 w-full h-full p-2'>
      <JobsHeader
        numberOfJobs={jobs.length}
        pipelineId={pipelineId}
        onRefresh={() => refetch()}
        onBack={handleBack}
      />
      <div className='h-full flex flex-col gap-2'>
        {jobs.length > 0 ? (
          <JobsTable jobs={jobs} pipelineId={pipelineId} />
        ) : (
          <div className='flex items-center justify-center h-full'>
            <div className='text-center space-y-2'>
              <p className='text-muted-foreground'>No jobs found</p>
              <p className='text-sm text-muted-foreground'>
                Run your pipeline to create the first job
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
