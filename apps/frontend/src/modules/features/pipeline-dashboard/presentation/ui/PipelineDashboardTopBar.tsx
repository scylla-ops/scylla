import { Button } from '@shadcn';
import { useNavigate } from 'react-router-dom';
import { Trash } from 'lucide-react';
import { useDeletePipeline } from '@/modules/features/pipeline-dashboard/presentation/hooks/useDeletePipeline.ts';
import { usePipelineDashboardStore } from '@/modules/features/pipeline-dashboard/presentation/stores/usePipelineDashboardStore.ts';

export const PipelineDashboardTopBar = () => {
  const navigate = useNavigate();
  const deletePipeline = useDeletePipeline();
  const selectedPipelineIds = usePipelineDashboardStore(state => state.selectedPipelineIds);
  const clearSelection = usePipelineDashboardStore(state => state.clearSelection);

  const deleteSelectedPipelines = () => {
    selectedPipelineIds.forEach(id => {
      deletePipeline.mutate(id);
    });
    clearSelection();
  };

  return (
    <div className={'flex items-center justify-end gap-2'}>
      {selectedPipelineIds.length > 0 && (
        <>
          <Button variant={'outline'} onClick={clearSelection}>
            Clear
          </Button>
          <Button
            size='icon'
            variant='destructive'
            onClick={() => {
              deleteSelectedPipelines();
            }}
            className='h-9 w-9'
          >
            <Trash className='size-4' />
          </Button>
        </>
      )}
      <Button onClick={() => navigate('pipeline-creation')}>New</Button>{' '}
    </div>
  );
};
