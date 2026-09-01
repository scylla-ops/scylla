import { Trans } from '@lingui/react/macro';
import { FeatureHeader } from '@shared/presentation/ui';
import { useFeatureSelection } from '@shared/presentation/hooks/use-feature-selection.ts';
import { useDeleteSecret } from '@/modules/features/secret/presentation/hooks/use-secrets.ts';
import { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { useCan } from '@/modules/features/permission/presentation/hooks/use-authorization.ts';

interface CredentialsHeaderProps {
  activeCount: number;
  secretIds: string[];
  onAddSecret?: () => void;
  projectId: string;
}

export const SecretHeader = ({
  activeCount,
  secretIds,
  onAddSecret,
  projectId,
}: CredentialsHeaderProps) => {
  const deleteSecret = useDeleteSecret(projectId);
  const { headerProps } = useFeatureSelection('secrets', secretIds, {
    deleteItem: id => deleteSecret.mutateAsync(id),
  });

  const canCreate = useCan(Permission.CREATE_SECRET, { projectId });
  const canDelete = useCan(Permission.DELETE_SECRET, { projectId });

  return (
    <FeatureHeader
      count={activeCount}
      label={<Trans>Secret</Trans>}
      pluralLabel={<Trans>Secrets</Trans>}
      onNew={onAddSecret}
      newLabel={<Trans>New secret</Trans>}
      canNew={canCreate}
      newDeniedReason={<Trans>You don't have permission to create secrets.</Trans>}
      canDelete={canDelete}
      deleteDeniedReason={<Trans>You don't have permission to delete secrets.</Trans>}
      {...headerProps}
    />
  );
};
