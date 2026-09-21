import { useParams } from 'react-router-dom';
import { usePipelinesMetadata } from '../../hooks/use-pipelines-metadata.ts';
import { useQueries } from '@tanstack/react-query';
import { jobsByPipelinesQueries } from '@/modules/features/jobs';
import { useAuthorization } from '@platform/authz';
import { ErrorState } from '@shared/presentation/ui/feedback/ErrorState.tsx';
import { Trans } from '@lingui/react/macro';
import { PaginationSlot } from '@shared/presentation/ui/data-display/Pagination.tsx';
import { PipelineDashboardHeader } from '@/modules/features/pipeline/presentation/ui/dashboard/PipelineDashboardHeader.tsx';
import { PipelineTable } from '@/modules/features/pipeline/presentation/ui/dashboard/pipeline-table/PipelineTable.tsx';

export const DashboardPipelinePage = () => {
  const { projectId } = useParams();
  const { pipelines, isError, errorMessage, paginationInfo, setPage, containerRef } =
    usePipelinesMetadata(projectId!);

  const pipelineIds = (pipelines?.items ?? []).map(p => p.id);
  // `jobs` is Svelte as of Phase 3, so what was `useJobsByPipelines` is now a
  // plain fan-out description `useQueries` runs unchanged. `useAuthorization`
  // is what subscribes this component to the permissions store — `can()` inside
  // the factory reads it, but only a hook makes React re-render when it lands.
  useAuthorization();
  const { queries, combine, canListJobs } = jobsByPipelinesQueries(pipelineIds);
  const { jobsByPipelineId, isJobsError, isJobsLoading } = useQueries({ queries, combine });

  if (isError) {
    return <ErrorState message={String(errorMessage) || 'Unable to load pipelines'} />;
  }

  // The frame renders before the pipelines do: the table area has to be in the
  // DOM for its height to be measured, and that height decides what to fetch.
  return (
    <div className='flex flex-col gap-4 w-full h-full min-h-0'>
      <PipelineDashboardHeader
        numberOfPipelines={paginationInfo?.totalCount ?? pipelineIds.length}
        pipelineIds={pipelineIds}
      />
      <div ref={containerRef} className='flex-1 min-h-0 overflow-auto'>
        <div className='relative'>
          {pipelines && pipelines.items.length > 0 && (
            <PipelineTable
              pipelines={pipelines.items}
              jobsByPipelineId={jobsByPipelineId}
              isJobsError={isJobsError}
              isJobsLoading={isJobsLoading}
              canListJobs={canListJobs}
            />
          )}
          {pipelines && pipelines.items.length === 0 && (
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
      <PaginationSlot paginationInfo={paginationInfo} onPageChange={setPage} />
    </div>
  );
};
