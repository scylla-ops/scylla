import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import { useFeatureSelection } from '@shared/presentation/hooks/use-feature-selection.ts';
import { Trans } from '@lingui/react/macro';
import { FeatureHeader } from '@shared/presentation/ui';
import { useDeletePipeline } from '@/modules/features/pipeline/presentation/hooks/use-delete-pipeline.ts';
import { Button } from '@shadcn';
import { KeyIcon, Users } from 'lucide-react';
import { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { useCan } from '@/modules/features/permission/presentation/hooks/use-authorization.ts';
import { Can } from '@/modules/features/permission/presentation/ui/authorization/Can.tsx';

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
        label={<Trans>Pipeline</Trans>}
        pluralLabel={<Trans>Pipelines</Trans>}
        {...headerProps}
        onNew={goToCreatePipeline}
        newLabel={<Trans>New pipeline</Trans>}
        canNew={canCreate}
        newDeniedReason={<Trans>You don't have permission to create pipelines.</Trans>}
        canDelete={canDelete}
        deleteDeniedReason={<Trans>You don't have permission to delete pipelines.</Trans>}
        extraActions={
          <>
            {/* The project dashboard is where someone stands when they think
                about who works on the project, so its member page opens from
                here rather than from a separate administration screen. */}
            <Can permission={Permission.LIST_PROJECT_MEMBERS}>
              <Button variant={'outline'} onClick={() => goToSubRoute('members')}>
                <Users className={'text-primary'} />
                <Trans>Members</Trans>
              </Button>
            </Can>
            <Can permission={Permission.LIST_SECRETS}>
              <Button variant={'outline'} onClick={() => goToSubRoute('secrets')}>
                <KeyIcon className={'text-primary'} />
                <Trans>Secrets</Trans>
              </Button>
            </Can>
          </>
        }
      />
    </div>
  );
};
