import { Button } from '@shadcn';
import { useNavigate } from 'react-router-dom';
import { Trash } from 'lucide-react';
import { useDeletePipeline } from '@/modules/features/pipeline-dashboard/presentation/hooks/useDeletePipeline.ts';
import { usePipelineDashboardStore } from '@/modules/features/pipeline-dashboard/presentation/stores/usePipelineDashboardStore.ts';
import { useState } from 'react';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/ConfirmOperationAlertDialog.tsx';

//TODO: put the delete pipeline action in a separate component
export const PipelineDashboardTopBar = () => {
  const navigate = useNavigate();
  const deletePipeline = useDeletePipeline();
  const selectedPipelineIds = usePipelineDashboardStore(state => state.selectedPipelineIds);
  const clearSelection = usePipelineDashboardStore(state => state.clearSelection);
  const [deleteDialogVisibility, setDeleteDialogVisibility] = useState(false);

  const handleDelete = async () => {
    try {
      const promises = selectedPipelineIds.map(id => deletePipeline.mutateAsync(id));

      await Promise.all(promises);
      setDeleteDialogVisibility(false);
      clearSelection();
    } catch (error) {
      console.log(error);
    }
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
              setDeleteDialogVisibility(true);
            }}
            className='h-9 w-9'
          >
            <Trash className='size-4' />
          </Button>
        </>
      )}
      <Button onClick={() => navigate(window.location.pathname + '/create')}>New pipeline</Button>{' '}
      <ConfirmOperationAlertDialog
        onContinue={handleDelete}
        open={deleteDialogVisibility}
        onOpenChange={setDeleteDialogVisibility}
      />
    </div>
  );
};
