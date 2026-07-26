import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import { useFeatureSelection } from '@shared/presentation/hooks/use-feature-selection.ts';
import { Trans } from '@lingui/react/macro';
import { FeatureHeader } from '@shared/presentation/ui';
import { useDeletePipeline } from '@/modules/features/pipeline/presentation/hooks/use-delete-pipeline.ts';
import { Button } from '@shadcn';
import { KeyIcon } from 'lucide-react';
import { Permission } from '@/modules/features/role/domain/structs/permission.struct.ts';
import { useCan } from '@/modules/features/role/presentation/hooks/use-authorization.ts';
import { Can } from '@/modules/features/role/presentation/ui/authorization/Can.tsx';

interface PipelineDashboardHeaderProps {
  numberOfPipelines: number;
  pipelineIds: string[];
}

export const PipelineDashboardHeader = ({
  numberOfPipelines,
  pipelineIds,
}: PipelineDashboardHeaderProps) => {
  const { goToCreatePipeline, goToSubRoute } = useScyllaNavigate();
  const deletePipeline = useDeletePipeline();
  const { headerProps } = useFeatureSelection('pipelines', pipelineIds, {
    deleteItem: id => deletePipeline.mutateAsync(id),
  });

  const canCreate = useCan(Permission.CREATE_PIPELINE);
  const canDelete = useCan(Permission.DELETE_PIPELINE);

  return (
    <div className='flex items-center gap-4 w-full'>
      <FeatureHeader
        count={numberOfPipelines}
        label={'Pipeline'}
        {...headerProps}
        onNew={goToCreatePipeline}
        newLabel={<Trans>New pipeline</Trans>}
        canNew={canCreate}
        newDeniedReason={<Trans>You don't have permission to create pipelines.</Trans>}
        canDelete={canDelete}
        deleteDeniedReason={<Trans>You don't have permission to delete pipelines.</Trans>}
        extraActions={
          <Can permission={Permission.LIST_SECRETS}>
            <Button variant={'outline'} onClick={() => goToSubRoute('secrets')}>
              <KeyIcon className={'text-primary'} />
              Secrets
            </Button>
          </Can>
        }
      />
    </div>
  );
};
