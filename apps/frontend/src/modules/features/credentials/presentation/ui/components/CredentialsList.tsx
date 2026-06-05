import { DataTable } from '@shared/presentation/ui/DataTable';
import { createCredentialsColumns } from './credentials-columns';
import type { Credential } from '@/modules/features/credentials/domain/models/credential.model.ts';

interface CredentialsListProps {
  credentials: Credential[];
}

export const CredentialsList = ({ credentials }: CredentialsListProps) => {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const columns = createCredentialsColumns() as any;

  return <DataTable columns={columns} data={credentials} getRowId={(row) => row.id} />;
};

