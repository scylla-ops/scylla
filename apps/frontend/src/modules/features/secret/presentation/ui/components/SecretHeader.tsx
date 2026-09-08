import { Trans } from '@lingui/react/macro';
import { FeatureHeader } from '@shared/presentation/ui';
import { Permission } from '@platform/authz';
import { useCan } from '@platform/authz';

interface CredentialsHeaderProps {
  activeCount: number;
  onAddSecret?: () => void;
  projectId: string;
}

export const SecretHeader = ({ activeCount, onAddSecret, projectId }: CredentialsHeaderProps) => {
  const canCreate = useCan(Permission.CREATE_SECRET, { projectId });

  return (
    <FeatureHeader
      count={activeCount}
      label={<Trans>Secret</Trans>}
      pluralLabel={<Trans>Secrets</Trans>}
      onNew={onAddSecret}
      newLabel={<Trans>New secret</Trans>}
      canNew={canCreate}
      newDeniedReason={<Trans>You don't have permission to create secrets.</Trans>}
    />
  );
};
