import { DataTable } from '@shared/presentation/ui/data-display/DataTable';
import { createCredentialsColumns } from './secret-columns.tsx';
import type { SecretEntity } from '@/modules/features/secret/domain/entities/secret.entity.ts';
import { useDeleteSecret } from '@/modules/features/secret/presentation/hooks/use-secrets.ts';
import { useFeatureSelection } from '@shared/presentation/hooks/use-feature-selection.ts';
import { Permission, useCan } from '@platform/authz';
import { toast } from '@shared/presentation/utils/toast.ts';
import { Trans, useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/feedback/ConfirmOperationAlertDialog.tsx';
import { useState } from 'react';

interface CredentialsListProps {
  secrets: SecretEntity[];
  projectId: string;
}

export const SecretList = ({ secrets, projectId }: CredentialsListProps) => {
  const deleteSecret = useDeleteSecret(projectId);
  const { selectedIds, select, headerProps } = useFeatureSelection(
    'secrets',
    secrets.map(secret => secret.id),
    { deleteItem: id => deleteSecret.mutateAsync(id) },
  );
  const canDelete = useCan(Permission.DELETE_SECRET, { projectId });
  const { i18n, t } = useLingui();

  const [selectedSecretId, setSelectedSecretId] = useState<string | null>(null);

  const handleDelete = (secretId: string) => {
    deleteSecret.mutate(secretId, {
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.SECRET_DELETE));
      },
    });
  };

  const columns = createCredentialsColumns({ onDelete: setSelectedSecretId });

  return (
    <>
      <DataTable
        isRowSelected={row => selectedIds.includes(row.id)}
        columns={columns}
        data={secrets}
        onRowClick={row => select(row.id)}
        getRowId={row => row.id}
        searchable
        searchPlaceholder={t`Search a secret…`}
        searchValues={secret => [secret.name, secret.description]}
        selection={{
          ...headerProps,
          canDelete,
          deleteDeniedReason: <Trans>You don't have permission to delete secrets.</Trans>,
        }}
        alignColumnsCenter
      />
      <ConfirmOperationAlertDialog
        open={selectedSecretId !== null}
        onOpenChange={value => {
          if (!value) setSelectedSecretId(null);
        }}
        onContinue={() => {
          if (!selectedSecretId) return;
          handleDelete(selectedSecretId);
          select(selectedSecretId);
          setSelectedSecretId(null);
        }}
      />
    </>
  );
};
