import { Button } from '@shadcn';
import { RefreshCw } from 'lucide-react';
import { useDeleteJobs } from '@/modules/features/jobs/presentation/hooks/use-delete-jobs.ts';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';
import { Trans } from '@lingui/react/macro';
import { FeatureHeader } from '@shared/presentation/ui';
import { useFeatureSelection } from '@shared/presentation/hooks/use-feature-selection.ts';
import { Permission } from '@platform/authz/domain/structs/permission.struct.ts';
import { useCan } from '@platform/authz/presentation/hooks/use-authorization.ts';

interface JobsHeaderProps {
  numberOfJobs: number;
  jobIds: string[];
  pipelineId: string;
  onRefresh: () => void;
  /**
   * Runs the pipeline these jobs belong to. Injected by whoever owns the route,
   * because running is a pipeline operation — jobs would otherwise have to
   * import the pipeline module that already reads jobs.
   */
  onRun?: () => Promise<void>;
}

export const JobsHeader = ({
  numberOfJobs,
  jobIds,
  pipelineId,
  onRefresh,
  onRun,
}: JobsHeaderProps) => {
  const deleteJob = useDeleteJobs(pipelineId);
  const { headerProps } = useFeatureSelection('jobs', jobIds, {
    deleteItem: id => deleteJob.mutateAsync(id),
  });

  const canRun = useCan(Permission.RUN_PIPELINE);
  const canDelete = useCan(Permission.DELETE_JOB);

  return (
    <div className={'flex flex-col gap-3'}>
      <FeatureHeader
        count={numberOfJobs}
        label={<Trans>Job</Trans>}
        pluralLabel={<Trans>Jobs</Trans>}
        newLabel={<Trans>Run</Trans>}
        onNew={onRun ? () => void onRun() : undefined}
        canNew={canRun}
        newDeniedReason={<Trans>You don't have permission to run this pipeline.</Trans>}
        canDelete={canDelete}
        deleteDeniedReason={<Trans>You don't have permission to delete jobs.</Trans>}
        {...headerProps}
        underLabel={
          <div className={'flex items-center justify-between'}>
            <div className='flex items-baseline gap-2'>
              <span className='text-sm text-muted-foreground font-medium'>
                <Trans>Pipeline ID: {pipelineId}</Trans>
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
