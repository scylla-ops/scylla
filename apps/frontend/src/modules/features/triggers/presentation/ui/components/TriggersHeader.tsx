import { Trans } from '@lingui/react/macro';
import { FeatureHeader } from '@shared/presentation/ui';
import { Permission } from '@platform/authz';
import { useCan } from '@platform/authz';

interface TriggersHeaderProps {
  count: number;
  pipelineId: string;
  onNew: () => void;
}

export const TriggersHeader = ({ count, pipelineId, onNew }: TriggersHeaderProps) => {
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
      underLabel={
        <span className='text-sm text-muted-foreground font-medium'>
          <Trans>Pipeline ID: {pipelineId}</Trans>
        </span>
      }
    />
  );
};
