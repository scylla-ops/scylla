import { Button } from '@shadcn';
import { Trash } from 'lucide-react';
import { useDeletePipeline } from '@/modules/features/pipeline-dashboard/presentation/hooks/useDeletePipeline.ts';
import { usePipelineDashboardStore } from '@/modules/features/pipeline-dashboard/presentation/stores/usePipelineDashboardStore.ts';
import { useState } from 'react';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/ConfirmOperationAlertDialog.tsx';
import { useScyllaNavigate } from '@shared/presentation/hooks/useScyllaNavigate.ts';

interface PipelineDashboardHeaderProps {
  numberOfPipelines: number;
}
//TODO: put the delete pipeline action in a separate component
export const PipelineDashboardHeader = ({ numberOfPipelines }: PipelineDashboardHeaderProps) => {
  const { goToCreatePipeline } = useScyllaNavigate();
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
    <div className={'flex flex-row items-center justify-between'}>
      <div className='flex items-baseline gap-2'>
        <h1 className='text-3xl font-bold tracking-tight'>
          <span className='text-primary'>{numberOfPipelines}</span>{' '}
          <span className='text-foreground'>
            Pipeline
            {numberOfPipelines > 1 ? 's' : ''}
          </span>
        </h1>
        <span className='text-sm text-muted-foreground font-medium'>in total</span>
      </div>
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
        <Button onClick={() => goToCreatePipeline()}>New pipeline</Button>{' '}
        <ConfirmOperationAlertDialog
          onContinue={handleDelete}
          open={deleteDialogVisibility}
          onOpenChange={setDeleteDialogVisibility}
        />
      </div>
    </div>
  );
};
