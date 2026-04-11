import { useParams } from 'react-router-dom';
import { usePipelines } from '../hooks/usePipelines.ts';
import { PipelineTable } from '@/modules/features/pipeline-dashboard/presentation/ui/pipeline-table/PipelineTable.tsx';
import { PipelineDashboardHeader } from '@/modules/features/pipeline-dashboard/presentation/ui/PipelineDashboardHeader.tsx';
import { Pagination } from '@/modules/shared/presentation/ui/Pagination.tsx';
import { ErrorState } from '@/modules/shared/presentation/ui/ErrorState.tsx';

export const DashboardPipelinePage = () => {
  const { projectId } = useParams();
  const { isLoading, pipelines, isError, errorMessage, paginationInfo, setPage } = usePipelines(
    projectId!,
  );

  if (isLoading || !pipelines) {
    return <></>;
  }

  if (isError) {
    return <ErrorState message={String(errorMessage) || 'Impossible de charger les pipelines'} />;
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
