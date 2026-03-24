import { usePipelines } from '../hooks/usePipelines.ts';
import { PipelineTable } from '@/modules/features/pipeline-dashboard/presentation/ui/pipeline-table/PipelineTable.tsx';

export const DashboardPipelinePage = () => {
  const { isLoading, pipelines, isError, errorMessage } = usePipelines();

  if (isLoading || !pipelines) {
    return <div className='flex items-center justify-center h-screen'>Loading pipelines...</div>;
  }

  if (isError) {
    return (
      <div className='flex items-center justify-center h-screen text-red-500'>
        Error: {String(errorMessage)}
      </div>
    );
  }

  return (
    <div className='h-full flex flex-col gap-2'>
      {pipelines.length > 0 ? (
        <PipelineTable pipelines={pipelines} />
      ) : (
        <div className='text-gray-500'>No pipelines found</div>
      )}
    </div>
  );
};
