import { Button } from '@shadcn';
import { Trash, RefreshCw, ArrowLeft } from 'lucide-react';
import { useDeleteJobs } from '@/modules/features/jobs/presentation/hooks/use-delete-jobs.ts';
import { useJobsStore } from '@/modules/features/jobs/presentation/stores/use-jobs.store.ts';
import { useState } from 'react';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/ConfirmOperationAlertDialog.tsx';

interface JobsHeaderProps {
  numberOfJobs: number;
  pipelineId: string;
  pipelineName?: string;
  onRefresh: () => void;
  onBack: () => void;
}

export const JobsHeader = ({
  numberOfJobs,
  pipelineId,
  pipelineName,
  onRefresh,
  onBack,
}: JobsHeaderProps) => {
  const deleteJob = useDeleteJobs(pipelineId);
  const selectedJobIds = useJobsStore(state => state.selectedJobIds);
  const clearSelection = useJobsStore(state => state.clearSelection);
  const [deleteDialogVisibility, setDeleteDialogVisibility] = useState(false);

  const handleDelete = async () => {
    try {
      const promises = selectedJobIds.map(id => deleteJob.mutateAsync(id));
      await Promise.all(promises);
      setDeleteDialogVisibility(false);
      clearSelection();
    } catch (error) {
      console.error('Error deleting jobs:', error);
    }
  };

  return (
    <div className={'flex flex-col gap-3'}>
      <div className='flex items-center gap-2'>
        <Button variant='ghost' size='icon' onClick={onBack} className='h-8 w-8'>
          <ArrowLeft className='size-4' />
        </Button>
        <div>
          <h1 className='text-3xl font-bold tracking-tight'>
            <span className='text-primary'>{numberOfJobs}</span>{' '}
            <span className='text-foreground'>Job{numberOfJobs > 1 ? 's' : ''}</span>
          </h1>
          {pipelineName && (
            <p className='text-sm text-muted-foreground'>
              Pipeline: <span className='font-medium'>{pipelineName}</span>
            </p>
          )}
        </div>
      </div>

      <div className={'flex items-center justify-between'}>
        <div className='flex items-baseline gap-2'>
          <span className='text-sm text-muted-foreground font-medium'>
            Pipeline ID: {pipelineId}
          </span>
        </div>
        <div className={'flex items-center justify-end gap-2'}>
          <Button variant={'outline'} size='icon' onClick={onRefresh} className='h-9 w-9'>
            <RefreshCw className='size-4' />
          </Button>

          {selectedJobIds.length > 0 && (
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
        </div>
        <ConfirmOperationAlertDialog
          onContinue={handleDelete}
          open={deleteDialogVisibility}
          onOpenChange={setDeleteDialogVisibility}
          title='Delete Jobs'
          description={`Are you sure you want to delete ${selectedJobIds.length} job(s)? This action cannot be undone.`}
        />
      </div>
    </div>
  );
};
