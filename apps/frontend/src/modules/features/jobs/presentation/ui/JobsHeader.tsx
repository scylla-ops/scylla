import { Button } from '@shadcn';
import { RefreshCw } from 'lucide-react';
import { useDeleteJobs } from '@/modules/features/jobs/presentation/hooks/use-delete-jobs.ts';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';
import { Trans } from '@lingui/react/macro';
import { FeatureHeader } from '@shared/presentation/ui';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';
import { useRunPipeline } from '@/modules/features/pipeline/presentation/hooks/use-run-pipeline.ts';

interface JobsHeaderProps {
  numberOfJobs: number;
  pipelineId: string;
  onRefresh: () => void;
  onBack: () => void;
}

export const JobsHeader = ({ numberOfJobs, pipelineId, onRefresh, onBack }: JobsHeaderProps) => {
  const deleteJob = useDeleteJobs(pipelineId);
  const { selectedIds, clearSelection } = useSelection('jobs');

  const runPipeline = useRunPipeline();

  const handleRunPipeline = async () => {
    try {
      await runPipeline.mutateAsync(pipelineId);
    } catch {
      // Toast shown by the global MutationCache onError handler.
    }
  };

  const handleDelete = async () => {
    const promises = selectedIds.map(id => deleteJob.mutateAsync(id));
    await Promise.allSettled(promises);
    clearSelection();
  };

  return (
    <div className={'flex flex-col gap-3'}>
      <FeatureHeader
        count={numberOfJobs}
        label={'Job'}
        pluralLabel={'Jobs'}
        onBack={onBack}
        newLabel={'Run'}
        onNew={handleRunPipeline}
        selectedCount={selectedIds.length}
        onClearSelection={clearSelection}
        onDeleteSelection={handleDelete}
        underLabel={
          <div className={'flex items-center justify-between'}>
            <div className='flex items-baseline gap-2'>
              <span className='text-sm text-muted-foreground font-medium'>
                Pipeline ID: {pipelineId}
              </span>
            </div>
          </div>
        }
        extraActions={
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant={'outline'}
                size='icon'
                onClick={onRefresh}
                className='h-9 w-9 cursor-pointer transition-all hover:scale-110'
              >
                <RefreshCw className='size-4' />
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>
                <Trans>Refresh</Trans>
              </p>
            </TooltipContent>
          </Tooltip>
        }
      />
    </div>
  );
};
