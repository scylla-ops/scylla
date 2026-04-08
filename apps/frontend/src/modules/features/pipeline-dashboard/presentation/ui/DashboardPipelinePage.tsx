import { useParams } from 'react-router-dom';
import { usePipelines } from '../hooks/usePipelines.ts';
import { PipelineTable } from '@/modules/features/pipeline-dashboard/presentation/ui/pipeline-table/PipelineTable.tsx';
import PipelineTableSkeleton from '@/modules/features/pipeline-dashboard/presentation/ui/pipeline-table/PipelineTableSkeleton.tsx';
import { Skeleton } from '@/modules/shared/presentation/ui/shadcn/skeleton.tsx';
import { useDelayedLoading } from '@/modules/shared/presentation/hooks/useDelayedLoading.ts';
import { PipelineDashboardHeader } from '@/modules/features/pipeline-dashboard/presentation/ui/PipelineDashboardHeader.tsx';
import { Pagination } from '@/modules/shared/presentation/ui/Pagination.tsx';

export const DashboardPipelinePage = () => {
  const { projectId } = useParams();
  const { isLoading, pipelines, isError, errorMessage, paginationInfo, setPage } = usePipelines(
    projectId!,
  );
  const showSkeleton = useDelayedLoading(400);

  if (isLoading && !showSkeleton) {
    return <></>;
  }

  if ((isLoading && showSkeleton) || !pipelines) {
    return (
      <div className='flex flex-col gap-4 w-full h-full p-2'>
        <div className='flex items-baseline gap-2'>
          <Skeleton className='h-9 w-56' />
          <Skeleton className='h-5 w-16' />
        </div>
        <div className='h-full flex flex-col gap-2'>
          <PipelineTableSkeleton />
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
            {String(errorMessage) || 'Impossible de charger les pipelines'}
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className='flex flex-col gap-4 w-full min-h-full p-2'>
      <PipelineDashboardHeader numberOfPipelines={paginationInfo?.totalCount ?? pipelines.length} />
      <div className='flex-1 flex flex-col gap-2'>
        {pipelines.length > 0 ? (
          <PipelineTable pipelines={pipelines} />
        ) : (
          <div className='flex items-center justify-center h-full'>
            <div className='text-center space-y-2'>
              <p className='text-muted-foreground'>No pipeline found</p>
              <p className='text-sm text-muted-foreground'>
                Create your first pipeline to get started
              </p>
            </div>
          </div>
        )}
      </div>
      {paginationInfo && paginationInfo.totalPages > 1 && (
        <Pagination paginationInfo={paginationInfo} onPageChange={setPage} className='pb-2' />
      )}
    </div>
  );
};
