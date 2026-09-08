import { Trans } from '@lingui/react/macro';
import { FeatureHeader } from '@shared/presentation/ui';
import { useFeatureSelection } from '@shared/presentation/hooks/use-feature-selection.ts';
import { useDeleteTrigger } from '@/modules/features/triggers/presentation/hooks/use-delete-trigger.ts';
import { Permission } from '@platform/authz';
import { useCan } from '@platform/authz';

interface TriggersHeaderProps {
  count: number;
  triggerIds: string[];
  pipelineId: string;
  onNew: () => void;
}

export const TriggersHeader = ({
  count,
  triggerIds,
  pipelineId,
  onNew,
}: TriggersHeaderProps) => {
  const deleteTrigger = useDeleteTrigger(pipelineId);
  const { headerProps } = useFeatureSelection('triggers', triggerIds, {
    deleteItem: id => deleteTrigger.mutateAsync(id),
  });

  // Triggers are all-or-nothing in V1: one permission covers create and delete.
  const canManage = useCan(Permission.MANAGE_TRIGGERS);

  return (
    <FeatureHeader
      count={count}
      label={<Trans>Trigger</Trans>}
      pluralLabel={<Trans>Triggers</Trans>}
      newLabel={<Trans>New trigger</Trans>}
      onNew={onNew}
      canNew={canManage}
      newDeniedReason={<Trans>You don't have permission to manage triggers.</Trans>}
      canDelete={canManage}
      deleteDeniedReason={<Trans>You don't have permission to manage triggers.</Trans>}
      {...headerProps}
      underLabel={
        <span className='text-sm text-muted-foreground font-medium'>
          <Trans>Pipeline ID: {pipelineId}</Trans>
        </span>
      }
    />
  );
};
