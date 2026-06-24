import { DataTable } from '@shared/presentation/ui/data-display/DataTable';
import { createCredentialsColumns } from './secret-columns.tsx';
import type { SecretEntity } from '@/modules/features/secret/domain/entities/secret.entity.ts';
import { useDeleteSecret } from '@/modules/features/secret/presentation/hooks/use-secrets.ts';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/feedback/ConfirmOperationAlertDialog.tsx';
import { useState } from 'react';

interface CredentialsListProps {
  secrets: SecretEntity[];
  projectId: string;
}

export const SecretList = ({ secrets, projectId }: CredentialsListProps) => {
  const deleteSecret = useDeleteSecret(projectId);
  const { selectedIds, select } = useSelection('secrets');
  const { i18n } = useLingui();

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
