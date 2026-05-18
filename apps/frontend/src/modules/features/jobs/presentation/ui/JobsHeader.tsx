import { Button } from '@shadcn';
import { Trash, RefreshCw } from 'lucide-react';
import { BackButton } from '@shared/presentation/ui/BackButton.tsx';
import { useDeleteJobs } from '@/modules/features/jobs/presentation/hooks/use-delete-jobs.ts';
import { useJobsStore } from '@/modules/features/jobs/presentation/stores/use-jobs.store.ts';
import { useState } from 'react';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/ConfirmOperationAlertDialog.tsx';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';
import { Trans } from '@lingui/react/macro';

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
    const promises = selectedJobIds.map(id => deleteJob.mutateAsync(id));
    await Promise.allSettled(promises);
    setDeleteDialogVisibility(false);
    clearSelection();
  };

  return (
    <div className={'flex flex-col gap-3'}>
      <div className='flex items-center gap-2'>
        <BackButton iconOnly onClick={onBack} />
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
          <Tooltip>
            <TooltipTrigger asChild>
              <Button variant={'outline'} size='icon' onClick={onRefresh} className='h-9 w-9 cursor-pointer transition-all hover:scale-110'>
                <RefreshCw className='size-4' />
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <p><Trans>Refresh</Trans></p>
            </TooltipContent>
          </Tooltip>

          {selectedJobIds.length > 0 && (
            <>
              <Button variant={'outline'} onClick={clearSelection}>
                Clear
              </Button>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    size='icon'
                    variant='destructive'
                    onClick={() => {
                      setDeleteDialogVisibility(true);
                    }}
                    className='h-9 w-9 cursor-pointer transition-all hover:scale-110'
                  >
                    <Trash className='size-4' />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  <p><Trans>Delete</Trans></p>
                </TooltipContent>
              </Tooltip>
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
