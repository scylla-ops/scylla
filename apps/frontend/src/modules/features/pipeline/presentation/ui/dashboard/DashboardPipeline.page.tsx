import { useParams } from 'react-router-dom';
import { usePipelinesMetadata } from '../../hooks/use-pipelines-metadata.ts';
import { usePipelineJobs } from '../../hooks/use-pipeline-jobs.ts';
import { ErrorState } from '@shared/presentation/ui/feedback/ErrorState.tsx';
import { Trans } from '@lingui/react/macro';
import { Pagination } from '@shared/presentation/ui/data-display/Pagination.tsx';
import { PipelineDashboardHeader } from '@/modules/features/pipeline/presentation/ui/dashboard/PipelineDashboardHeader.tsx';
import { PipelineTable } from '@/modules/features/pipeline/presentation/ui/dashboard/pipeline-table/PipelineTable.tsx';

export const DashboardPipelinePage = () => {
  const { projectId } = useParams();
  const { isLoading, pipelines, isError, errorMessage, paginationInfo, setPage } =
    usePipelinesMetadata(projectId!);

  const pipelineIds = (pipelines?.items ?? []).map(p => p.id);
  const { jobsByPipelineId, isJobsError, isJobsLoading } = usePipelineJobs(pipelineIds);

  if (isLoading || !pipelines) {
    return <></>;
  }

  if (isError) {
    return <ErrorState message={String(errorMessage) || 'Unable to load pipelines'} />;
  }

  return (
    <div className='flex flex-col gap-4 w-full h-full'>
      <PipelineDashboardHeader
        numberOfPipelines={paginationInfo?.totalCount ?? pipelines.items.length}
        pipelineIds={pipelineIds}
      />
      <div className='flex-1 min-h-0 overflow-auto'>
        <div className='relative'>
          {pipelines.items.length > 0 ? (
            <PipelineTable
              pipelines={pipelines.items}
              jobsByPipelineId={jobsByPipelineId}
              isJobsError={isJobsError}
              isJobsLoading={isJobsLoading}
            />
          ) : (
            <div className='flex items-center justify-center h-full min-h-100'>
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
