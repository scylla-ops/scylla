import { DataTable } from '@shared/presentation/ui/DataTable';
import { createCredentialsColumns } from './secret-columns.tsx';
import type { Secret } from '@/modules/features/secret/domain/models/secret.model.ts';
import { useDeleteSecret } from '@/modules/features/secret/presentation/hooks/use-secrets.ts';

interface CredentialsListProps {
  secrets: Secret[];
  projectId: string;
}

export const SecretList = ({ secrets, projectId }: CredentialsListProps) => {
  const deleteSecret = useDeleteSecret(projectId);

  const handleDelete = (secretId: string) => deleteSecret.mutate(secretId);

  const columns = createCredentialsColumns({ onDelete: handleDelete });

  return <DataTable columns={columns} data={secrets} getRowId={row => row.id} alignColumnsCenter />;
};
