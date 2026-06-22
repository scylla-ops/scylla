import { Button } from '@shadcn';
import { RefreshCw } from 'lucide-react';
import { useDeleteJobs } from '@/modules/features/jobs/presentation/hooks/use-delete-jobs.ts';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';
import { Trans } from '@lingui/react/macro';
import { FeatureHeader } from '@shared/presentation/ui';
import { useFeatureSelection } from '@shared/presentation/hooks/use-feature-selection.ts';
import { useRunPipeline } from '@/modules/features/pipeline/presentation/hooks/use-run-pipeline.ts';

interface JobsHeaderProps {
  numberOfJobs: number;
  jobIds: string[];
  pipelineId: string;
  onRefresh: () => void;
}

export const JobsHeader = ({ numberOfJobs, jobIds, pipelineId, onRefresh }: JobsHeaderProps) => {
  const deleteJob = useDeleteJobs(pipelineId);
  const { headerProps } = useFeatureSelection('jobs', jobIds, {
    deleteItem: id => deleteJob.mutateAsync(id),
  });

  const runPipeline = useRunPipeline();

  const handleRunPipeline = async () => {
    try {
      await runPipeline.mutateAsync(pipelineId);
    } catch {
      // Toast shown by the global MutationCache onError handler.
    }
  };

  return (
    <div className={'flex flex-col gap-3'}>
      <FeatureHeader
        count={numberOfJobs}
        label={'Job'}
        pluralLabel={'Jobs'}
        newLabel={'Run'}
        onNew={handleRunPipeline}
        {...headerProps}
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
