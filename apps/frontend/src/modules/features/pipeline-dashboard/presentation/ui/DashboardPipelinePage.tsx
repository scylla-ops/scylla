import { StatusCard } from './StatusCard.tsx';
import { usePipelines } from '../hooks/usePipelines.ts';
import { usePipelineDashboardStore } from '@/modules/features/pipeline-dashboard/presentation/stores/usePipelineDashboardStore.ts';
import { ListCard, type ListCardSection } from '@shared/presentation/ui';

export const DashboardPipelinePage = () => {
  const { isLoading, pipelines, isError, errorMessage } = usePipelines();
  const selectPipeline = usePipelineDashboardStore(state => state.selectPipeline);
  const selectedPipelineIds = usePipelineDashboardStore(state => state.selectedPipelineIds);

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

  const columns: ListCardSection[] = [
    {
      width: '20%',
      className: 'flex justify-center items-center gap-4 shrink-0 text-slate-500 text-sm',
      content: (
        <>
          <span className='text-xs italic truncate'>Status</span>
        </>
      ),
    },
    {
      width: '35%',
      className: 'flex justify-center items-center gap-4 shrink-0 text-slate-500 text-sm',
      content: (
        <>
          <span className='text-xs italic truncate'>History</span>
        </>
      ),
    },
    {
      width: '20%',
      className: 'flex justify-center items-center gap-4 shrink-0 text-slate-500 text-sm',
      content: (
        <>
          <span className='text-xs italic truncate'>Last execution</span>
        </>
      ),
    },
    {
      width: '20%',
      className: 'flex flex-1 justify-center items-center gap-4 shrink-0 text-slate-500 text-sm',
      content: (
        <>
          <span className='text-xs italic truncate'>Actions</span>
        </>
      ),
    },
  ];

  return (
    <div className={'flex flex-col h-full gap-3'}>
      <ListCard sections={columns} className='px-4 py-2 mb-4' />
      <div className='h-full flex flex-col gap-2'>
        {pipelines.length > 0 ? (
          pipelines.map((pipeline, index) => (
            <StatusCard
              key={index}
              selected={selectedPipelineIds.includes(pipeline.pipelineId)}
              onClick={() => {
                selectPipeline(pipeline.pipelineId);
              }}
              pipeline={pipeline}
            />
          ))
        ) : (
          <div className='text-gray-500'>No pipelines found</div>
        )}
      </div>
    </div>
  );
};
