import { useParams } from 'react-router-dom';
import { usePipelines } from '../hooks/usePipelines.ts';
import { PipelineTable } from '@/modules/features/pipeline-dashboard/presentation/ui/pipeline-table/PipelineTable.tsx';
import { PipelineDashboardHeader } from '@/modules/features/pipeline-dashboard/presentation/ui/PipelineDashboardHeader.tsx';
import { ErrorState } from '@/modules/shared/presentation/ui/ErrorState.tsx';
import { Trans } from '@lingui/react/macro';
import { Pagination } from '@shared/presentation/ui/Pagination.tsx';

export const DashboardPipelinePage = () => {
  const { projectId } = useParams();
  const { isLoading, pipelines, isError, errorMessage, paginationInfo, setPage } = usePipelines(
    projectId!,
  );

  if (isLoading || !pipelines) {
    return <></>;
  }

  if (isError) {
    return <ErrorState message={String(errorMessage) || 'Unable to load pipelines'} />;
  }

  return (
    <div className='flex flex-col gap-4 w-full h-full p-4'>
      <PipelineDashboardHeader numberOfPipelines={paginationInfo?.totalCount ?? pipelines.length} />
      <div className='flex-1 min-h-0 overflow-auto'>
        <div className='relative'>
          {pipelines.length > 0 ? (
            <PipelineTable pipelines={pipelines} />
          ) : (
            <div className='flex items-center justify-center h-full min-h-[400px]'>
              <div className='text-center space-y-2'>
                <p className='text-muted-foreground'>
                  <Trans>No pipeline found</Trans>
                </p>
                <p className='text-sm text-muted-foreground'>
                  <Trans>Create your first pipeline to get started</Trans>
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
