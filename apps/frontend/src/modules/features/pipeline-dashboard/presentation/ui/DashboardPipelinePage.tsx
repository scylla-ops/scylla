import { StatusCard } from './StatusCard.tsx';
import { usePipelines } from '../hooks/usePipelines.ts';

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
    <>
      <div className='grid grid-cols-1 md:grid-cols-3 gap-4 w-full'>
        {pipelines.length > 0 ? (
          pipelines.map((pipeline, index) => <StatusCard key={index} pipeline={pipeline} />)
        ) : (
          <div className='text-gray-500'>No pipelines found</div>
        )}
      </div>
    </>
  );
};
