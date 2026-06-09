import { DataTable } from '@shared/presentation/ui/DataTable';
import { createCredentialsColumns } from './secret-columns.tsx';
import type { Secret } from '@/modules/features/secret/domain/models/secret.model.ts';
import { useDeleteSecret } from '@/modules/features/secret/presentation/hooks/use-secrets.ts';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/ConfirmOperationAlertDialog.tsx';
import { useState } from 'react';

interface CredentialsListProps {
  secrets: Secret[];
  projectId: string;
}

export const SecretList = ({ secrets, projectId }: CredentialsListProps) => {
  const deleteSecret = useDeleteSecret(projectId);
  const { selectedIds, select } = useSelection('secrets');

  const [selectedSecretId, setSelectedSecretId] = useState<string | null>(null);

  const handleDelete = (secretId: string) => {
    deleteSecret.mutate(secretId, {
      onSuccess: () => {
        toast.success('Secret deleted');
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
